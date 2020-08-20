use chrono::{DateTime, Utc};
use client::PocketClient;
use errors::PocketError;
use futures::TryFutureExt;
use hyper::http::uri::InvalidUri;
use hyper::Uri;
use serde::{Deserialize, Serialize};
use serialization::*;
use std::convert::TryInto;
use std::result::Result;
use url::Url;
use add::*;
use send::*;
use get::*;

pub mod add;
pub mod auth;
pub mod get;
pub mod send;
mod client;
pub mod errors;
mod headers;
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
pub struct ItemImage {
    #[serde(deserialize_with = "from_str")]
    pub item_id: u64,
    #[serde(default, deserialize_with = "try_url_from_string")]
    pub src: Option<Url>,
    #[serde(deserialize_with = "from_str")]
    pub width: u16,
    #[serde(deserialize_with = "from_str")]
    pub height: u16,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct DomainMetaData {
    pub name: Option<String>,
    pub logo: String,
    pub greyscale_logo: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct ItemTag {
    #[serde(deserialize_with = "from_str")]
    pub item_id: u64,
    pub tag: String,
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


#[derive(Deserialize, Debug, PartialEq)]
pub struct PocketSearchMeta {
    search_type: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum PocketItemStatus {
    #[serde(rename = "0")]
    Normal,
    #[serde(rename = "1")]
    Archived,
    #[serde(rename = "2")]
    Deleted,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct PocketItem {
    #[serde(deserialize_with = "from_str")]
    pub item_id: u64,

    #[serde(default, deserialize_with = "try_url_from_string")]
    pub given_url: Option<Url>,
    pub given_title: String,

    #[serde(deserialize_with = "from_str")]
    pub word_count: usize,
    pub excerpt: String,

    #[serde(with = "string_date_unix_timestamp_format")]
    pub time_added: DateTime<Utc>,
    #[serde(deserialize_with = "option_string_date_unix_timestamp_format")]
    pub time_read: Option<DateTime<Utc>>,
    #[serde(with = "string_date_unix_timestamp_format")]
    pub time_updated: DateTime<Utc>,
    #[serde(deserialize_with = "option_string_date_unix_timestamp_format")]
    pub time_favorited: Option<DateTime<Utc>>,

    #[serde(deserialize_with = "bool_from_int_string")]
    pub favorite: bool,

    #[serde(deserialize_with = "bool_from_int_string")]
    pub is_index: bool,
    #[serde(deserialize_with = "bool_from_int_string")]
    pub is_article: bool,
    pub has_image: PocketItemHas,
    pub has_video: PocketItemHas,

    #[serde(deserialize_with = "from_str")]
    pub resolved_id: u64,
    pub resolved_title: String,
    #[serde(default, deserialize_with = "try_url_from_string")]
    pub resolved_url: Option<Url>,

    pub sort_id: u64,

    pub status: PocketItemStatus,
    #[serde(default, deserialize_with = "optional_vec_from_map")]
    pub tags: Option<Vec<ItemTag>>,
    #[serde(default, deserialize_with = "optional_vec_from_map")]
    pub images: Option<Vec<PocketImage>>,
    #[serde(default, deserialize_with = "optional_vec_from_map")]
    pub videos: Option<Vec<ItemVideo>>,
    #[serde(default, deserialize_with = "optional_vec_from_map")]
    pub authors: Option<Vec<ItemAuthor>>,
    pub lang: String,
    pub time_to_read: Option<u64>,
    pub domain_metadata: Option<DomainMetaData>,
    pub listen_duration_estimate: Option<u64>,
    pub image: Option<ItemImage>,
    #[serde(default, deserialize_with = "try_url_from_string")]
    pub amp_url: Option<Url>,
    #[serde(default, deserialize_with = "try_url_from_string")]
    pub top_image_url: Option<Url>,
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