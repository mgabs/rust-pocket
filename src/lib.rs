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

pub mod add;
pub mod auth;
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

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PocketGetRequest<'a> {
    search: Option<&'a str>,
    domain: Option<&'a str>,

    tag: Option<PocketGetTag<'a>>,
    state: Option<PocketGetState>,
    content_type: Option<PocketGetType>,
    detail_type: Option<PocketGetDetail>,
    #[serde(serialize_with = "optional_bool_to_int")]
    favorite: Option<bool>,

    #[serde(serialize_with = "optional_datetime_to_int")]
    since: Option<DateTime<Utc>>,

    sort: Option<PocketGetSort>,
    #[serde(serialize_with = "optional_to_string")]
    count: Option<usize>,
    #[serde(serialize_with = "optional_to_string")]
    offset: Option<usize>,
}

impl<'a> PocketGetRequest<'a> {
    pub fn new() -> PocketGetRequest<'a> {
        Default::default()
    }

    pub fn search<'b>(&'b mut self, search: &'a str) -> &'b mut PocketGetRequest<'a> {
        self.search = Some(search);
        self
    }

    pub fn domain<'b>(&'b mut self, domain: &'a str) -> &'b mut PocketGetRequest<'a> {
        self.domain = Some(domain);
        self
    }

    pub fn tag<'b>(&'b mut self, tag: PocketGetTag<'a>) -> &'b mut PocketGetRequest<'a> {
        self.tag = Some(tag);
        self
    }

    pub fn state<'b>(&'b mut self, state: PocketGetState) -> &'b mut PocketGetRequest<'a> {
        self.state = Some(state);
        self
    }

    pub fn content_type<'b>(
        &'b mut self,
        content_type: PocketGetType,
    ) -> &'b mut PocketGetRequest<'a> {
        self.content_type = Some(content_type);
        self
    }

    pub fn detail_type<'b>(
        &'b mut self,
        detail_type: PocketGetDetail,
    ) -> &'b mut PocketGetRequest<'a> {
        self.detail_type = Some(detail_type);
        self
    }

    pub fn complete<'b>(&'b mut self) -> &'b mut PocketGetRequest<'a> {
        self.detail_type(PocketGetDetail::Complete)
    }

    pub fn simple<'b>(&'b mut self) -> &'b mut PocketGetRequest<'a> {
        self.detail_type(PocketGetDetail::Simple)
    }

    pub fn archived<'b>(&'b mut self) -> &'b mut PocketGetRequest<'a> {
        self.state(PocketGetState::Archive)
    }

    pub fn unread<'b>(&'b mut self) -> &'b mut PocketGetRequest<'a> {
        self.state(PocketGetState::Unread)
    }

    pub fn articles<'b>(&'b mut self) -> &'b mut PocketGetRequest<'a> {
        self.content_type(PocketGetType::Article)
    }

    pub fn videos<'b>(&'b mut self) -> &'b mut PocketGetRequest<'a> {
        self.content_type(PocketGetType::Video)
    }

    pub fn images<'b>(&'b mut self) -> &'b mut PocketGetRequest<'a> {
        self.content_type(PocketGetType::Image)
    }

    pub fn favorite<'b>(&'b mut self, fav: bool) -> &'b mut PocketGetRequest<'a> {
        self.favorite = Some(fav);
        self
    }

    pub fn since<'b>(&'b mut self, since: DateTime<Utc>) -> &'b mut PocketGetRequest<'a> {
        self.since = Some(since);
        self
    }

    pub fn sort<'b>(&'b mut self, sort: PocketGetSort) -> &'b mut PocketGetRequest<'a> {
        self.sort = Some(sort);
        self
    }

    pub fn sort_by_newest<'b>(&'b mut self) -> &'b mut PocketGetRequest<'a> {
        self.sort(PocketGetSort::Newest)
    }

    pub fn sort_by_oldest<'b>(&'b mut self) -> &'b mut PocketGetRequest<'a> {
        self.sort(PocketGetSort::Oldest)
    }

    pub fn sort_by_title<'b>(&'b mut self) -> &'b mut PocketGetRequest<'a> {
        self.sort(PocketGetSort::Title)
    }

    pub fn sort_by_site<'b>(&'b mut self) -> &'b mut PocketGetRequest<'a> {
        self.sort(PocketGetSort::Site)
    }

    pub fn offset<'b>(&'b mut self, offset: usize) -> &'b mut PocketGetRequest<'a> {
        self.offset = Some(offset);
        self
    }

    pub fn count<'b>(&'b mut self, count: usize) -> &'b mut PocketGetRequest<'a> {
        self.count = Some(count);
        self
    }

    pub fn slice<'b>(&'b mut self, offset: usize, count: usize) -> &'b mut PocketGetRequest<'a> {
        self.offset(offset).count(count)
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PocketGetDetail {
    Simple,
    Complete,
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PocketGetSort {
    Newest,
    Oldest,
    Title,
    Site,
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PocketGetState {
    Unread,
    Archive,
    All,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum PocketGetTag<'a> {
    #[serde(serialize_with = "untagged_to_str")]
    Untagged,
    Tagged(&'a str),
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PocketGetType {
    Article,
    Video,
    Image,
}

#[derive(Deserialize, Debug, PartialEq)]
struct PocketSearchMeta {
    search_type: String,
}

#[derive(Deserialize, Debug, PartialEq)]
struct PocketGetResponse {
    #[serde(deserialize_with = "vec_from_map")]
    list: Vec<PocketItem>,
    status: u16,
    #[serde(deserialize_with = "bool_from_int")]
    complete: bool,
    error: Option<String>,
    search_meta: PocketSearchMeta,
    #[serde(deserialize_with = "int_date_unix_timestamp_format")]
    since: DateTime<Utc>,
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
    use chrono::TimeZone;
    use serde::Serialize;
    use utils::remove_whitespace;

    // Get
    // PocketGetRequest
    #[test]
    fn test_serialize_get_request() {
        let request = &PocketGetRequest {
            search: Some("search"),
            domain: Some("domain"),

            tag: Some(PocketGetTag::Untagged),
            state: Some(PocketGetState::All),
            content_type: Some(PocketGetType::Article),
            detail_type: Some(PocketGetDetail::Complete),
            favorite: Some(false),
            since: Some(Utc::now()),

            sort: Some(PocketGetSort::Newest),
            count: Some(1),
            offset: Some(2),
        };

        let actual = serde_json::to_string(request).unwrap();

        let expected = remove_whitespace(&format!(
            r#"
                    {{
                        "search": "{search}",
                        "domain": "{domain}",
                        "tag": "{tag}",
                        "state": "{state}",
                        "contentType": "{content_type}",
                        "detailType": "{detail_type}",
                        "favorite": "{favorite}",
                        "since": "{since}",
                        "sort": "{sort}",
                        "count": "{count}",
                        "offset": "{offset}"
                    }}
               "#,
            search = request.search.unwrap(),
            domain = request.domain.unwrap(),
            tag = to_inner_json_string(&request.tag.as_ref()),
            state = to_inner_json_string(&request.state.unwrap()),
            content_type = to_inner_json_string(&request.content_type.unwrap()),
            detail_type = to_inner_json_string(&request.detail_type.unwrap()),
            favorite = if request.favorite.unwrap() { 1 } else { 0 },
            since = request.since.unwrap().timestamp().to_string(),
            sort = to_inner_json_string(&request.sort.unwrap()),
            count = request.count.unwrap(),
            offset = request.offset.unwrap(),
        ));

        assert_eq!(actual, expected);
    }

    fn to_inner_json_string<T: Serialize>(value: T) -> String {
        serde_json::to_value(value)
            .unwrap()
            .as_str()
            .unwrap()
            .trim_matches('\"')
            .to_string()
    }

    // PocketGetDetail
    // PocketGetSort
    // PocketGetState
    // PocketGetTag
    // PocketGetType
    // ItemVideo
    // PocketItemHas

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

    // PocketSendAction
    // PocketSendRequest
    // PocketSendResponse

  

    // PocketGetResponse
    #[test]
    fn test_deserialize_get_response_with_list_map() {
        let expected = PocketGetResponse {
            list: vec![],
            status: 1,
            complete: true,
            error: None,
            search_meta: PocketSearchMeta {
                search_type: "normal".to_string(),
            },
            since: Utc.timestamp(1584221353, 0),
        };
        let response = remove_whitespace(&format!(
            r#"
                    {{
                        "status": {status},
                        "complete": {complete},
                        "list": {{}},
                        "error": null,
                        "search_meta": {{
                            "search_type": "{search_type}"
                        }},
                        "since": {since}
                    }}
               "#,
            status = expected.status,
            complete = if expected.complete { 1 } else { 0 },
            search_type = expected.search_meta.search_type,
            since = expected.since.timestamp(),
        ));

        let actual: PocketGetResponse = serde_json::from_str(&response).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_deserialize_get_response_with_list_array() {
        let expected = PocketGetResponse {
            list: vec![],
            status: 2,
            complete: true,
            error: None,
            search_meta: PocketSearchMeta {
                search_type: "normal".to_string(),
            },
            since: Utc.timestamp(1584221353, 0),
        };
        let response = remove_whitespace(&format!(
            r#"
                {{
                    "status": {status},
                    "complete": {complete},
                    "list": [],
                    "error": null,
                    "search_meta": {{
                        "search_type": "{search_type}"
                    }},
                    "since": {since}
                }}
           "#,
            status = expected.status,
            complete = if expected.complete { 1 } else { 0 },
            search_type = expected.search_meta.search_type,
            since = expected.since.timestamp(),
        ));

        let actual: PocketGetResponse = serde_json::from_str(&response).unwrap();

        assert_eq!(actual, expected);
    }

    // PocketItemStatus
    // PocketItem
}
