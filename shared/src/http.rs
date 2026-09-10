use crate::error::AppError;

pub enum HttpMethod {
    Get,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
        }
    }
}

pub enum HttpCacheMode {
    Default,
    Reload,
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub cache_mode: HttpCacheMode,
}

pub struct Response {
    pub status: u16,
    pub bytes: Vec<u8>,
}

impl Response {
    pub fn is_success(&self) -> bool {
        (200..=299).contains(&self.status)
    }
}

/// What a platform must supply for the loader to reach the artifact repository. The web implements it over
/// the browser's fetch; a native client over an HTTP client of its own. Not `Send`: the browser's response
/// handles are not, and one trait serves every platform.
#[allow(async_fn_in_trait)]
pub trait HttpFetch {
    async fn fetch(&self, request: &HttpRequest) -> Result<Response, AppError>;
}
