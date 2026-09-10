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

/// The transport the artifact loader reaches the repository through. The returned future is deliberately
/// not `Send`: some platforms' response handles are not, and one trait serves every platform.
#[allow(async_fn_in_trait)]
pub trait HttpFetch {
    async fn fetch(&self, request: &HttpRequest) -> Result<Response, AppError>;
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeMap;

    use super::*;

    /// Serves the bodies it was seeded with and 404s everything else, recording each URL asked for so a
    /// test can assert on what was and was not requested.
    pub(crate) struct MockHttpFetch {
        bodies_by_url: BTreeMap<String, Vec<u8>>,
        requested_urls: tokio::sync::Mutex<Vec<String>>,
    }

    impl MockHttpFetch {
        pub(crate) fn new(bodies_by_url: BTreeMap<String, Vec<u8>>) -> MockHttpFetch {
            MockHttpFetch {
                bodies_by_url,
                requested_urls: tokio::sync::Mutex::new(Vec::new()),
            }
        }

        pub(crate) async fn requested_urls(&self) -> Vec<String> {
            self.requested_urls.lock().await.clone()
        }
    }

    impl HttpFetch for MockHttpFetch {
        async fn fetch(&self, request: &HttpRequest) -> Result<Response, AppError> {
            self.requested_urls.lock().await.push(request.url.clone());

            let body: Option<&Vec<u8>> = self.bodies_by_url.get(&request.url);

            match body {
                Some(bytes) => Ok(Response {
                    status: 200,
                    bytes: bytes.clone(),
                }),
                None => Ok(Response {
                    status: 404,
                    bytes: Vec::new(),
                }),
            }
        }
    }

    #[tokio::test]
    async fn mock_fetch_returns_404_for_an_unseeded_url() {
        let http_fetch: MockHttpFetch = MockHttpFetch::new(BTreeMap::new());

        let response: Response = http_fetch
            .fetch(&HttpRequest {
                method: HttpMethod::Get,
                url: "https://repository.example/absent".to_string(),
                cache_mode: HttpCacheMode::Default,
            })
            .await
            .unwrap();

        assert!(!response.is_success());
    }
}
