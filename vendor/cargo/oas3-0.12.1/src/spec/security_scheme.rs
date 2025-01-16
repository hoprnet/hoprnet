use serde::{Deserialize, Serialize};

use super::Flows;

/// Defines a security scheme that can be used by the operations.
///
/// Supported schemes are HTTP authentication, an API key (either as a header or as a query
/// parameter), OAuth2's common flows (implicit, password, application and access code) as defined
/// in [RFC6749], and [OpenID Connect Discovery].
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/HEAD/versions/3.1.0.md#security-scheme-object>.
///
/// [RFC6749]: https://tools.ietf.org/html/rfc6749
/// [OpenID Connect Discovery]: https://tools.ietf.org/html/draft-ietf-oauth-discovery-06
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum SecurityScheme {
    #[serde(rename = "apiKey")]
    ApiKey {
        /// A description for security scheme.
        ///
        /// CommonMark syntax MAY be used for rich text representation.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        name: String,
        #[serde(rename = "in")]
        location: String,
    },

    #[serde(rename = "http")]
    Http {
        /// A description for security scheme.
        ///
        /// CommonMark syntax MAY be used for rich text representation.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        scheme: String,
        #[serde(rename = "bearerFormat")]
        bearer_format: Option<String>,
    },

    #[serde(rename = "oauth2")]
    OAuth2 {
        /// A description for security scheme.
        ///
        /// CommonMark syntax MAY be used for rich text representation.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        flows: Flows,
    },

    #[serde(rename = "openIdConnect")]
    OpenIdConnect {
        /// A description for security scheme.
        ///
        /// CommonMark syntax MAY be used for rich text representation.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        #[serde(rename = "openIdConnectUrl")]
        open_id_connect_url: String,
    },

    #[serde(rename = "mutualTLS")]
    MutualTls {
        /// A description for security scheme.
        ///
        /// CommonMark syntax MAY be used for rich text representation.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[test]
    fn test_http_basic_deser() {
        const HTTP_BASIC_SAMPLE: &str = r#"{"type": "http", "scheme": "basic"}"#;
        let obj: SecurityScheme = serde_json::from_str(HTTP_BASIC_SAMPLE).unwrap();

        assert!(matches!(
            obj,
            SecurityScheme::Http {
                description: None,
                scheme,
                bearer_format: None,
            } if scheme == "basic"
        ));
    }

    #[test]
    fn test_security_scheme_oauth_deser() {
        const IMPLICIT_OAUTH2_SAMPLE: &str = r#"{
          "type": "oauth2",
          "flows": {
            "implicit": {
              "authorizationUrl": "https://example.com/api/oauth/dialog",
              "scopes": {
                "write:pets": "modify pets in your account",
                "read:pets": "read your pets"
              }
            },
            "authorizationCode": {
              "authorizationUrl": "https://example.com/api/oauth/dialog",
              "tokenUrl": "https://example.com/api/oauth/token",
              "scopes": {
                "write:pets": "modify pets in your account",
                "read:pets": "read your pets"
              }
            }
          }
        }"#;

        let obj: SecurityScheme = serde_json::from_str(IMPLICIT_OAUTH2_SAMPLE).unwrap();
        match obj {
            SecurityScheme::OAuth2 {
                description: _,
                flows,
            } => {
                assert!(flows.implicit.is_some());
                let implicit = flows.implicit.unwrap();
                assert_eq!(
                    implicit.authorization_url,
                    Url::parse("https://example.com/api/oauth/dialog").unwrap()
                );
                assert!(implicit.scopes.contains_key("write:pets"));
                assert!(implicit.scopes.contains_key("read:pets"));

                assert!(flows.authorization_code.is_some());
                let auth_code = flows.authorization_code.unwrap();
                assert_eq!(
                    auth_code.authorization_url,
                    Url::parse("https://example.com/api/oauth/dialog").unwrap()
                );
                assert_eq!(
                    auth_code.token_url,
                    Url::parse("https://example.com/api/oauth/token").unwrap()
                );
                assert!(implicit.scopes.contains_key("write:pets"));
                assert!(implicit.scopes.contains_key("read:pets"));
            }
            _ => panic!("wrong security scheme type"),
        }
    }
}
