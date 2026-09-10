use serde::{Deserialize, Serialize};

use crate::artifact::schema_version;
use crate::error::AppError;

pub const DISCOVERY_SCHEMA_VERSION: u32 = 1;

/// The single forever-URL of the Eafora system. Consumers commit to exactly this
/// URL; everything else (including `repository_base_url`) is server-supplied at
/// runtime by the discovery document fetched from here.
pub const DISCOVERY_URL: &str = "https://app.eafora.org/discovery";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryDocument {
    pub schema_version: u32,
    pub repository_base_url: String,
    pub minimum_client_version: String,
    pub sunset: Option<String>,
}

pub fn parse_discovery_document(bytes: &[u8]) -> Result<DiscoveryDocument, AppError> {
    schema_version::require_schema_version(bytes, "schema_version", DISCOVERY_SCHEMA_VERSION)?;

    let document: DiscoveryDocument = serde_json::from_slice(bytes)?;

    Ok(document)
}


/// Which repository base a client should trust. `Static` covers both a discovery document that could not be
/// read and one that names the base already compiled in, so a caller that has already begun fetching against
/// the static base need not start again.
pub enum AuthoritativeBase {
    Static,
    Discovered(String),
}

pub fn authoritative_repository_base(
    static_base: &str,
    discovery: Result<DiscoveryDocument, AppError>,
) -> AuthoritativeBase {
    match discovery {
        Err(_) => AuthoritativeBase::Static,
        Ok(document) if document.repository_base_url == static_base => AuthoritativeBase::Static,
        Ok(document) => AuthoritativeBase::Discovered(document.repository_base_url),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_DISCOVERY_JSON: &str = r#"{
  "schema_version": 1,
  "repository_base_url": "https://repository.eafora.org",
  "minimum_client_version": "1.0.0",
  "sunset": null
}"#;

    #[test]
    fn parse_discovery_document_round_trips_fixture() {
        let document: DiscoveryDocument = parse_discovery_document(VALID_DISCOVERY_JSON.as_bytes()).unwrap();

        assert_eq!(document.schema_version, 1);
        assert_eq!(document.repository_base_url, "https://repository.eafora.org");
        assert_eq!(document.minimum_client_version, "1.0.0");
        assert_eq!(document.sunset, None);
    }

    #[test]
    fn parse_discovery_document_rejects_unknown_schema_version() {
        let json: String = VALID_DISCOVERY_JSON.replace("\"schema_version\": 1", "\"schema_version\": 2");

        let error: AppError = parse_discovery_document(json.as_bytes()).unwrap_err();

        assert!(error.to_string().contains("schema_version 2 comes from a newer build"));
    }

    #[test]
    fn parse_discovery_document_handles_missing_sunset_field() {
        let json: &str = r#"{
  "schema_version": 1,
  "repository_base_url": "https://repository.eafora.org",
  "minimum_client_version": "1.0.0"
}"#;

        let document: DiscoveryDocument = parse_discovery_document(json.as_bytes()).unwrap();

        assert_eq!(document.sunset, None);
    }

    #[test]
    fn parse_discovery_document_ignores_unknown_fields() {
        let json: &str = r#"{
  "schema_version": 1,
  "repository_base_url": "https://repository.eafora.org",
  "minimum_client_version": "1.0.0",
  "sunset": null,
  "field_added_in_a_future_v1_revision": "ignored"
}"#;

        let document: DiscoveryDocument = parse_discovery_document(json.as_bytes()).unwrap();

        assert_eq!(document.repository_base_url, "https://repository.eafora.org");
    }

    #[test]
    fn authoritative_repository_base_uses_static_when_discovery_fails() {
        let result: AuthoritativeBase = authoritative_repository_base(
            "/repository",
            Err(AppError::from("discovery failed".to_string())),
        );

        assert!(matches!(result, AuthoritativeBase::Static));
    }

    #[test]
    fn authoritative_repository_base_uses_static_when_discovery_matches() {
        let document: DiscoveryDocument = DiscoveryDocument {
            schema_version: 1,
            repository_base_url: "/repository".to_string(),
            minimum_client_version: "0.1.0".to_string(),
            sunset: None,
        };

        let result: AuthoritativeBase = authoritative_repository_base("/repository", Ok(document));

        assert!(matches!(result, AuthoritativeBase::Static));
    }

    #[test]
    fn authoritative_repository_base_uses_discovered_when_base_differs() {
        let document: DiscoveryDocument = DiscoveryDocument {
            schema_version: 1,
            repository_base_url: "https://repository.eafora.org".to_string(),
            minimum_client_version: "0.1.0".to_string(),
            sunset: None,
        };

        let result: AuthoritativeBase = authoritative_repository_base("/repository", Ok(document));

        match result {
            AuthoritativeBase::Discovered(url) => {
                assert_eq!(url, "https://repository.eafora.org");
            }
            AuthoritativeBase::Static => panic!(),
        }
    }
}
