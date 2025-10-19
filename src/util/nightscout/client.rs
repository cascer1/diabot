use crate::util::nightscout::v1_models::{CombinedNightscout, Status};
use crate::util::nightscout::v2_models::NightscoutV2Properties;
use reqwest::Url;
use reqwest::header::HeaderMap;
use std::fmt;
use thiserror::Error;
use tracing::{debug, error, info, trace};

pub type Result<T> = std::result::Result<T, NsError>;

/// Error type for the wrapper.
#[derive(Debug, Error)]
pub enum NsError {
    Http(#[from] reqwest::Error),

    Url(#[from] url::ParseError),

    InvalidTokenParse(#[from] reqwest::header::InvalidHeaderValue),

    Unauthorized {
        endpoint: String,
    },

    HttpStatus {
        status: reqwest::StatusCode,
        url: Url,
    },

    Json {
        endpoint: String,
        #[source]
        source: serde_json::Error,
    },
}

impl NsError {
    /// Returns true if the error message may include sensitive information, like a full URL.
    ///
    /// # Context
    /// Some errors (like `reqwest::Error`) include the full request URL in their `Display` output.
    /// We can't remove the URL because:
    /// - `reqwest::Error::without_url()` requires a mutable reference
    /// - `Display` only has access to `&self`
    /// - `reqwest::Error` doesn't implement `Clone` or `Copy`
    ///
    /// This method helps callers decide whether the error can be shown publicly
    /// or should be kept away from public view.
    pub fn is_sensitive(&self) -> bool {
        matches!(self, NsError::Http(_))
    }
}

impl fmt::Display for NsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NsError::Http(err) => {
                write!(
                    f,
                    "An internal error occurred while creating the request to the Nightscout instance: `{err}`\nPlease report this issue."
                )
            }
            NsError::Url(error) => {
                write!(
                    f,
                    "The provided URL is invalid: {error}. Make sure it starts with `https://` and is a valid Nightscout instance."
                )
            }
            NsError::InvalidTokenParse(_) => {
                write!(f, "The provided token could not be parsed.")
            }
            NsError::Unauthorized { endpoint } => {
                write!(
                    f,
                    "Unauthorized when accessing `{endpoint}`. The instance may require an access token."
                )
            }
            NsError::HttpStatus { status, url } => {
                let path = url.path();
                match *status {
                    reqwest::StatusCode::NOT_FOUND => write!(
                        f,
                        "The endpoint `{path}` was not found. This may not be a valid Nightscout instance."
                    ),
                    reqwest::StatusCode::FORBIDDEN => write!(f, "Access to `{path}` is forbidden."),
                    reqwest::StatusCode::BAD_REQUEST => write!(f, "Bad request sent to `{path}`."),
                    _ => write!(
                        f,
                        "Received unexpected status code `{status}` from `{path}`."
                    ),
                }
            }
            NsError::Json { endpoint, source } => {
                write!(
                    f,
                    "Failed to parse the response from `{endpoint}`: `{source}`\nThis may be an issue in Diabot, please report this if you continue to see this message."
                )
            }
        }
    }
}

/// A thin async client for interacting with a Nightscout instance's API.
#[derive(Clone, Debug)]
pub struct NightscoutClient {
    base: Url,
    client: reqwest::Client,
}

impl NightscoutClient {
    /// Creates a new client for a given Nightscout URL and optional API secret.
    ///
    /// The base URL should not include `/api/v1` or other paths.
    ///
    /// # Arguments
    /// - `base` - The base URL (`"https://my-nightscout.fly.dev"`)
    /// - `auth` - An optional api secret (not recommended) or token
    pub fn new(base: &str, auth: Option<&str>) -> Result<Self> {
        let base = Url::parse(base.trim_end_matches('/'))?;
        let mut builder = reqwest::Client::builder();

        if let Some(secret) = auth {
            let mut headers = HeaderMap::new();
            headers.insert("api-secret", secret.parse()?);
            builder = builder.default_headers(headers);
        }

        let client = builder.build()?;
        Ok(NightscoutClient { base, client })
    }

    pub fn new_unauthed(base: &str) -> Result<Self> {
        let (base, auth) = parse_nightscout_url(base)?;
        Self::new(&base, auth.as_deref())
    }

    fn api_v1_path(&self, path: &str) -> Result<Url> {
        let joined = self.base.join(&format!("api/v1/{path}"))?;
        Ok(joined)
    }

    fn api_v2_path(&self, path: &str) -> Result<Url> {
        let joined = self.base.join(&format!("api/v2/{path}"))?;
        Ok(joined)
    }

    async fn status_error(resp: reqwest::Response) -> NsError {
        let status = resp.status();
        let url = resp.url().clone();
        let text = resp.text().await.unwrap_or_default();
        let body_len = text.len();
        let snippet = if body_len > 500 {
            format!("{}…", &text[..500])
        } else {
            text
        };
        error!(
            http_status = %status,
            url = %url,
            body_len = body_len,
            body_snippet = %snippet,
            "Unexpected HTTP response from Nightscout"
        );
        if status == reqwest::StatusCode::UNAUTHORIZED {
            NsError::Unauthorized {
                endpoint: url.path().to_string(),
            }
        } else {
            NsError::HttpStatus { status, url }
        }
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: Url, label: &str) -> Result<T> {
        debug!("Loading {} URL {}", label, url);
        let resp = self.client.get(url.clone()).send().await?;
        if resp.status().is_client_error() || resp.status().is_server_error() {
            return Err(Self::status_error(resp).await);
        }

        debug!("Got success response for {}", label);
        let text = resp.text().await?;
        serde_json::from_str::<T>(&text).map_err(|e| {
            let len = text.len();
            let snippet = if len > 500 {
                format!("{}…", &text[..500])
            } else {
                text
            };
            error!(url = %url, error = %e, label = %label, body_len = len, body_snippet = %snippet, "Failed to decode JSON");
            NsError::Json { endpoint: label.to_string(), source: e }
        })
    }

    /// GET `/api/v2/properties/:comma_separated_list`
    pub async fn get_v2_properties(&self, props: &[&str]) -> Result<NightscoutV2Properties> {
        let path = format!("properties/{}", props.join(","));
        let url = self.api_v2_path(&path)?;
        self.get_json::<NightscoutV2Properties>(url, "properties")
            .await
    }

    /// GET /status
    pub async fn get_status(&self) -> Result<Status> {
        let url = self.api_v1_path("status.json")?;
        self.get_json::<Status>(url, "status").await
    }

    /// Fetches status and properties concurrently and returns them as `CombinedNightscout`.
    ///
    /// Errors if any of the endpoints fail.
    pub async fn fetch_combined(&self) -> Result<CombinedNightscout> {
        #[rustfmt::skip]
        let (status, properties) = tokio::try_join!(
            self.get_status(),
            self.get_v2_properties(&[]),
        )?;

        Ok(CombinedNightscout { status, properties })
    }
}

pub fn parse_nightscout_url(input: &str) -> Result<(String, Option<String>)> {
    let mut url_str = input.to_string();

    if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
        debug!(
            "Missing scheme in Nightscout URL: {}, adding https://",
            url_str
        );
        url_str = format!("https://{url_str}");
    }

    let parsed = Url::parse(&url_str)?;
    let mut final_url = parsed.clone();

    // Extract and remove the token
    let token = parsed
        .query_pairs()
        .find(|(k, _)| k == "token")
        .map(|(_, v)| v.to_string());

    final_url
        .query_pairs_mut()
        .clear()
        .extend_pairs(parsed.query_pairs().filter(|(k, _)| k != "token"));

    debug!("Input (after token removal): {}", final_url);

    // Trim trailing empty segments
    let mut path_segments = parsed.path_segments().map_or(Vec::new(), Iterator::collect);

    while path_segments.last() == Some(&"") {
        path_segments.pop();
    }

    if path_segments.ends_with(&["api", "v1"]) {
        debug!(
            "Removing API declaration in NS URL path: {:?}",
            path_segments
        );
        path_segments.truncate(path_segments.len() - 2);
    }

    // Rebuild the path
    final_url.set_path("");
    for segment in &path_segments {
        final_url.path_segments_mut().unwrap().push(segment);
    }

    debug!("Final URL: {}", final_url);

    Ok((final_url.to_string(), token))
}
