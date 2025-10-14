use crate::util::nightscout::v1_models::{CombinedNightscout, Status};
use crate::util::nightscout::v2_models::NightscoutV2Properties;
use reqwest::Url;
use reqwest::header::HeaderMap;
use thiserror::Error;
use tracing::{debug, error, info, trace};

pub type Result<T> = std::result::Result<T, NsError>;

/// Error type for the wrapper.
#[derive(Debug, Error)]
pub enum NsError {
    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),

    #[error("Can't parse token: {0}")]
    InvalidTokenParse(#[from] reqwest::header::InvalidHeaderValue),

    #[error("Unauthorized on endpoint {endpoint}")]
    Unauthorized { endpoint: String },

    #[error("Unexpected HTTP {status} from {url}")]
    HttpStatus {
        status: reqwest::StatusCode,
        url: Url,
    },

    #[error("Failed to decode {endpoint} JSON: {source}")]
    Json {
        endpoint: String,
        #[source]
        source: serde_json::Error,
    },
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
    /// - `base` - The base URL ("https://my-nightscout.fly.dev")
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
        let joined = self.base.join(&format!("/api/v1/{}", path))?;
        Ok(joined)
    }

    fn api_v2_path(&self, path: &str) -> Result<Url> {
        let joined = self.base.join(&format!("/api/v2/{}", path))?;
        Ok(joined)
    }

    async fn handle_unexpected(resp: reqwest::Response) -> NsError {
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
        if resp.status().is_success() {
            debug!("Got success response for {}", label);
            let text = resp.text().await?;
            serde_json::from_str::<T>(&text).map_err(|e| {
                let body_len = text.len();
                let snippet = if body_len > 500 {
                    format!("{}…", &text[..500])
                } else {
                    text
                };
                error!(url = %url, error = %e, label = %label, body_len = body_len, body_snippet = %snippet, "Failed to decode JSON");
                NsError::Json { endpoint: label.to_string(), source: e }
            })
        } else {
            Err(Self::handle_unexpected(resp).await)
        }
    }

    /// GET /api/v2/properties/:comma_separated_list
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

    /// Fetch status and properties concurrently and return a CombinedNightscout struct.
    pub async fn fetch_combined(&self) -> Result<CombinedNightscout> {
        #[rustfmt::skip]
        let (status_res, props_res) = tokio::join!(
            self.get_status(),
            self.get_v2_properties(&[]),
        );

        let status = status_res?;
        let properties = props_res?;

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
        url_str = format!("https://{}", url_str);
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
    let mut path_segments: Vec<&str> = parsed
        .path_segments()
        .map_or(Vec::new(), |segments| segments.collect());

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
