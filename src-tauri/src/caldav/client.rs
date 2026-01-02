use reqwest::{Client, Response, StatusCode};
use thiserror::Error;

use super::discovery;
use super::types::{AccountTestResult, CalendarInfo};

#[derive(Error, Debug)]
pub enum CalDavError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Server returned {status}: {message}")]
    ServerError { status: u16, message: String },

    #[error("Authentication failed")]
    AuthFailed,

    #[error("XML parsing error: {0}")]
    XmlParse(String),

    #[error("Discovery failed: {0}")]
    Discovery(String),
}

pub type Result<T> = std::result::Result<T, CalDavError>;

/// CalDAV client for a specific server.
pub struct CalDavClient {
    http: Client,
    pub base_url: String,
    username: String,
    password: String,
}

impl CalDavClient {
    /// Create a new CalDAV client.
    pub fn new(base_url: impl Into<String>, username: impl Into<String>, password: impl Into<String>) -> Result<Self> {
        let http = Client::builder()
            .redirect(reqwest::redirect::Policy::none()) // Handle redirects manually for well-known
            .build()?;

        let mut base_url = base_url.into();
        // Ensure base URL doesn't have trailing slash
        if base_url.ends_with('/') {
            base_url.pop();
        }

        Ok(Self {
            http,
            base_url,
            username: username.into(),
            password: password.into(),
        })
    }

    /// Make a PROPFIND request to the given path.
    pub async fn propfind(&self, path: &str, depth: u8, body: &str) -> Result<String> {
        let url = self.resolve_url(path);

        let response = self
            .http
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .basic_auth(&self.username, Some(&self.password))
            .header("Content-Type", "application/xml; charset=utf-8")
            .header("Depth", depth.to_string())
            .body(body.to_string())
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a GET request for well-known discovery (follows redirects manually).
    pub async fn get_well_known(&self) -> Result<Option<String>> {
        let url = format!("{}/.well-known/caldav", self.base_url);

        let response = self
            .http
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;

        // Check for redirect
        if response.status().is_redirection() {
            if let Some(location) = response.headers().get("location") {
                return Ok(Some(location.to_str().unwrap_or_default().to_string()));
            }
        }

        // 404 means well-known not supported, which is okay
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        // Any other error
        if !response.status().is_success() {
            return Err(CalDavError::ServerError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        Ok(None)
    }

    /// Resolve a path to a full URL.
    fn resolve_url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            path.to_string()
        } else if path.starts_with('/') {
            // Extract base (scheme + host + port) from base_url
            if let Ok(url) = reqwest::Url::parse(&self.base_url) {
                let host = url.host_str().unwrap_or("");
                let port_part = url.port().map(|p| format!(":{}", p)).unwrap_or_default();
                format!("{}://{}{}{}", url.scheme(), host, port_part, path)
            } else {
                format!("{}{}", self.base_url, path)
            }
        } else {
            format!("{}/{}", self.base_url, path)
        }
    }

    /// Handle HTTP response, checking for errors.
    async fn handle_response(&self, response: Response) -> Result<String> {
        let status = response.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(CalDavError::AuthFailed);
        }

        if !status.is_success() && status != StatusCode::MULTI_STATUS {
            let message = response.text().await.unwrap_or_default();
            return Err(CalDavError::ServerError {
                status: status.as_u16(),
                message,
            });
        }

        Ok(response.text().await?)
    }

    /// Test connection and discover CalDAV endpoints.
    pub async fn test_connection(&self) -> AccountTestResult {
        match self.discover().await {
            Ok((principal_url, calendar_home_url, calendars)) => {
                AccountTestResult::success(principal_url, calendar_home_url, calendars)
            }
            Err(e) => AccountTestResult::failure(e.to_string()),
        }
    }

    /// Run full CalDAV discovery.
    async fn discover(&self) -> Result<(String, String, Vec<CalendarInfo>)> {
        discovery::discover(self).await
    }
}
