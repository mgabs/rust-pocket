use add::*;
use client::PocketClient;
use errors::PocketError;
use futures::TryFutureExt;
use get::*;
use hyper::{http::uri::InvalidUri, Uri};
use send::*;
use serde::{Deserialize, Serialize};
use serialization::*;
use std::{convert::TryInto, result::Result};
use url::Url;

pub mod add;
pub mod auth;
mod client;
pub mod errors;
pub mod get;
mod headers;
pub mod send;
mod serialization;
mod utils;

pub type PocketResult<T> = Result<T, PocketError>;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct PocketImage {
    #[serde(deserialize_with = "from_str")]
    pub item_id: u64,
    #[serde(deserialize_with = "from_str")]
    pub image_id: u64,
    #[serde(default, deserialize_with = "try_url_from_string")]
    pub src: Option<Url>,
    #[serde(deserialize_with = "from_str")]
    pub width: u16,
    #[serde(deserialize_with = "from_str")]
    pub height: u16,
    pub credit: String,
    pub caption: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct ItemVideo {
    #[serde(deserialize_with = "from_str")]
    pub item_id: u64,
    #[serde(deserialize_with = "from_str")]
    pub video_id: u64,
    #[serde(default, deserialize_with = "try_url_from_string")]
    pub src: Option<Url>,
    #[serde(deserialize_with = "from_str")]
    pub width: u16,
    #[serde(deserialize_with = "from_str")]
    pub height: u16,
    #[serde(deserialize_with = "option_from_str")]
    pub length: Option<usize>,
    pub vid: String,
    #[serde(rename = "type", deserialize_with = "from_str")]
    pub vtype: u16,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct ItemAuthor {
    #[serde(deserialize_with = "from_str")]
    pub item_id: u64,
    #[serde(deserialize_with = "from_str")]
    pub author_id: u64,
    pub name: String,
    pub url: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum PocketItemHas {
    #[serde(rename = "0")]
    No,
    #[serde(rename = "1")]
    Yes,
    #[serde(rename = "2")]
    Is,
}

#[derive(Serialize)]
pub struct PocketUserRequest<'a, T> {
    consumer_key: &'a str,
    access_token: &'a str,
    #[serde(flatten)]
    request: T,
}

pub struct Pocket {
    consumer_key: String,
    access_token: String,
    client: PocketClient,
}

impl Pocket {
    pub fn new(consumer_key: &str, access_token: &str) -> Pocket {
        Pocket {
            consumer_key: consumer_key.to_string(),
            access_token: access_token.to_string(),
            client: PocketClient::new(),
        }
    }

    #[inline]
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub async fn add(&self, request: &PocketAddRequest<'_>) -> PocketResult<PocketAddedItem> {
        let body = &PocketUserRequest {
            consumer_key: &*self.consumer_key,
            access_token: &*self.access_token,
            request,
        };

        self.client
            .post("https://getpocket.com/v3/add", &body)
            .map_ok(|v: PocketAddResponse| v.item)
            .await
    }

    pub async fn get(&self, request: &PocketGetRequest<'_>) -> PocketResult<Vec<PocketItem>> {
        let body = &PocketUserRequest {
            consumer_key: &*self.consumer_key,
            access_token: &*self.access_token,
            request,
        };

        self.client
            .post("https://getpocket.com/v3/get", &body)
            .map_ok(|v: PocketGetResponse| v.list)
            .await
    }

    pub async fn send(&self, request: &PocketSendRequest<'_>) -> PocketResult<PocketSendResponse> {
        let data = serde_json::to_string(request.actions)?;
        let params = &[
            ("consumer_key", &*self.consumer_key),
            ("access_token", &*self.access_token),
            ("actions", &data),
        ];

        let url = Url::parse_with_params("https://getpocket.com/v3/send", params).unwrap();

        self.client.get(url_to_uri(&url).unwrap()).await
    }

    pub fn filter(&self) -> PocketGetRequest {
        PocketGetRequest::new()
    }
}

fn url_to_uri(url: &Url) -> Result<Uri, InvalidUri> {
    url.as_str().try_into()
}

#[cfg(test)]
mod test {
    use super::*;
    use utils::remove_whitespace;

    // ItemImage
    #[test]
    fn test_deserialize_item_image() {
        let expected = ItemImage {
            item_id: 1,
            src: Url::parse("http://localhost").ok(),
            width: 3,
            height: 4,
        };
        let response = remove_whitespace(&format!(
            r#"
                    {{
                        "item_id": "{item_id}",
                        "src": "{src}",
                        "width": "{width}",
                        "height": "{height}"
                    }}
               "#,
            item_id = expected.item_id,
            src = expected.src.as_ref().unwrap(),
            width = expected.width,
            height = expected.height,
        ));

        let actual: ItemImage = serde_json::from_str(&response).unwrap();

        assert_eq!(actual, expected);
    }
}
