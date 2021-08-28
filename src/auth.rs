use crate::{client::PocketClient, errors::PocketError, Pocket, PocketResult};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize)]
pub struct PocketOAuthRequest<'a> {
    consumer_key: &'a str,
    redirect_uri: &'a str,
    state: Option<&'a str>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct PocketOAuthResponse {
    code: String,
    state: Option<String>,
}

#[derive(Serialize)]
pub struct PocketAuthorizeRequest<'a> {
    consumer_key: &'a str,
    code: &'a str,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct PocketAuthorizeResponse {
    access_token: String,
    username: String,
    state: Option<String>,
}

pub struct PocketAuthentication {
    consumer_key: String,
    redirect_uri: String,
    client: PocketClient,
}

impl PocketAuthentication {
    pub fn new(consumer_key: &str, redirect_uri: &str) -> PocketAuthentication {
        PocketAuthentication {
            consumer_key: consumer_key.to_string(),
            redirect_uri: redirect_uri.to_string(),
            client: PocketClient::new(),
        }
    }

    pub async fn request(&self, state: Option<&str>) -> PocketResult<String> {
        let body = &PocketOAuthRequest {
            consumer_key: &self.consumer_key,
            redirect_uri: &self.redirect_uri,
            state,
        };

        self.client
            .post("https://getpocket.com/v3/oauth/request", &body)
            .await
            .and_then(|r: PocketOAuthResponse| {
                PocketAuthentication::verify_state(state, r.state.as_deref()).map(|()| r.code)
            })
    }

    fn verify_state(request_state: Option<&str>, response_state: Option<&str>) -> PocketResult<()> {
        match (request_state, response_state) {
            (Some(s1), Some(s2)) if s1 == s2 => Ok(()),
            (None, None) => Ok(()),
            _ => Err(PocketError::Proto(0, "State does not match".to_string())),
        }
    }

    pub fn authorize_url(&self, code: &str) -> Url {
        let params = vec![
            ("request_token", code),
            ("redirect_uri", &self.redirect_uri),
        ];
        let mut url = Url::parse("https://getpocket.com/auth/authorize").unwrap();
        url.query_pairs_mut().extend_pairs(params.into_iter());
        url
    }

    pub async fn authorize(&self, code: &str, state: Option<&str>) -> PocketResult<PocketUser> {
        let body = &PocketAuthorizeRequest {
            consumer_key: &self.consumer_key,
            code,
        };

        self.client
            .post("https://getpocket.com/v3/oauth/authorize", &body)
            .await
            .and_then(|r: PocketAuthorizeResponse| {
                PocketAuthentication::verify_state(state, r.state.as_deref()).map(|()| PocketUser {
                    consumer_key: self.consumer_key.clone(),
                    access_token: r.access_token,
                    username: r.username,
                })
            })
    }
}

#[derive(Debug)]
pub struct PocketUser {
    pub consumer_key: String,
    pub access_token: String,
    pub username: String,
}

// TODO - change this to a Into and move to Pocket
impl PocketUser {
    pub fn pocket(self) -> Pocket {
        Pocket::new(&self.consumer_key, &self.access_token)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::remove_whitespace;

    #[test]
    fn test_serialize_auth_request() {
        let request = &PocketOAuthRequest {
            consumer_key: "consumer_key",
            redirect_uri: "http://localhost",
            state: Some("state"),
        };

        let actual = serde_json::to_string(request).unwrap();

        let expected = remove_whitespace(&format!(
            r#"
                    {{
                        "consumer_key": "{consumer_key}",
                        "redirect_uri": "{redirect_uri}",
                        "state": "{state}"
                    }}
               "#,
            consumer_key = request.consumer_key,
            redirect_uri = request.redirect_uri,
            state = request.state.unwrap()
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_deserialize_auth_response() {
        let expected = PocketOAuthResponse {
            code: "code".to_string(),
            state: Some("state".to_string()),
        };
        let response = remove_whitespace(&format!(
            r#"
                    {{
                        "code": "{code}",
                        "state": "{state}"
                    }}
               "#,
            code = expected.code,
            state = expected.state.as_ref().unwrap()
        ));

        let actual: PocketOAuthResponse = serde_json::from_str(&response).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_authorize_request() {
        let request = &PocketAuthorizeRequest {
            consumer_key: "consumer_key",
            code: "code",
        };

        let actual = serde_json::to_string(request).unwrap();

        let expected = remove_whitespace(&format!(
            r#"
                    {{
                        "consumer_key": "{consumer_key}",
                        "code": "{code}"
                    }}
               "#,
            consumer_key = request.consumer_key,
            code = request.code
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_deserialize_authorize_response() {
        let expected = PocketAuthorizeResponse {
            access_token: "access_token".to_string(),
            username: "username".to_string(),
            state: None,
        };
        let response = remove_whitespace(&format!(
            r#"
                    {{
                        "access_token": "{access_token}",
                        "username": "{username}"
                    }}
               "#,
            access_token = expected.access_token,
            username = expected.username
        ));

        let actual: PocketAuthorizeResponse = serde_json::from_str(&response).unwrap();

        assert_eq!(actual, expected);
    }
}
