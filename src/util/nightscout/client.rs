use crate::util::nightscout::v2_models::NightscoutV2Properties;
use reqwest::header::HeaderMap;
use reqwest::Url;
use thiserror::Error;
use tracing::{debug, info};
use crate::util::nightscout::v1_models::{CombinedNightscout, Status};

pub type Result<T> = std::result::Result<T, NsError>;

/// Error type for the wrapper.
#[derive(Debug, Error)]
pub enum NsError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),

    #[error("Invalid token: {0}")]
    InvalidApiSecret(#[from] reqwest::header::InvalidHeaderValue),

    #[error("Other error: {0}")]
    Other(String),
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

    async fn handle_unexpected(resp: reqwest::Response) -> Result<NsError> {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Ok(NsError::Other(format!("Unexpected HTTP {}: {}", status, text)))
    }

    /// GET /api/v2/properties/:comma_separated_list
    pub async fn get_v2_properties(&self, props: &[&str]) -> Result<NightscoutV2Properties> {
        let path = format!("properties/{}", props.join(","));
        let url = self.api_v2_path(&path)?;
        info!("Loading URL {}", url);
        let resp = self.client.get(url).send().await?;
        if resp.status().is_success() {
            info!("Got success response for properties");
            Ok(resp.json::<NightscoutV2Properties>().await?)
        } else {
            Err(Self::handle_unexpected(resp).await?)
        }
    }

    /// GET /status
    pub async fn get_status(&self) -> Result<Status> {
        let url = self.api_v1_path("status.json")?;
        info!("Loading URL {}", url);
        let resp = self.client.get(url).send().await?;
        if resp.status().is_success() {
            info!("Got success response for status");
            Ok(resp.json::<Status>().await?)
        } else {
            Err(Self::handle_unexpected(resp).await?)
        }
    }

    /// Fetch status and properties concurrently and return a CombinedNightscout struct.
    pub async fn fetch_combined(&self) -> Result<CombinedNightscout> {
        let (status_res, props_res) = tokio::join!(
            self.get_status(),
            self.get_v2_properties(&[]),
        );

        let status = status_res?;
        let properties = props_res?;

        Ok(CombinedNightscout {
            status,
            properties,
        })
    }
}

pub fn parse_nightscout_url(input: &str) -> Result<(String, Option<String>)> {
    let mut url_str = input.to_string();

    if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
        debug!("Missing scheme in Nightscout URL: {}, adding https://", url_str);
        url_str = format!("https://{}", url_str);
    }

    let parsed = Url::parse(&url_str)?;
    let mut final_url = parsed.clone();

    // Extract and remove the token
    let token = parsed.query_pairs()
        .find(|(k, _)| k == "token")
        .map(|(_, v)| v.to_string());

    final_url.query_pairs_mut().clear().extend_pairs(
        parsed.query_pairs().filter(|(k, _)| k != "token")
    );

    debug!("Input (after token removal): {}", final_url);

    // Trim trailing empty segments
    let mut path_segments: Vec<&str> = parsed
        .path_segments()
        .map_or(Vec::new(), |segments| segments.collect());

    while path_segments.last() == Some(&"") {
        path_segments.pop();
    }

    if path_segments.ends_with(&["api", "v1"]) {
        debug!("Removing API declaration in NS URL path: {:?}", path_segments);
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
