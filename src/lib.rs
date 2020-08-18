extern crate chrono;
extern crate hyper;
extern crate hyper_native_tls;
extern crate url;

extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use chrono::{DateTime, TimeZone, Utc};
use hyper::client::{Client, IntoUrl, RequestBuilder};
use hyper::error::Error as HttpError;
use hyper::header::parsing::from_one_raw_str;
use hyper::header::{ContentType, Header, HeaderFormat};
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use hyper::mime::Mime;
use serde::de::{DeserializeOwned, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::BTreeMap;
use std::convert::From;
use std::error::Error;
use std::fmt::Display;
use std::io::Error as IoError;
use std::io::Read;
use std::result::Result;
use std::str::FromStr;
use url::Url;

#[derive(Debug)]
pub enum PocketError {
    Http(HttpError),
    Json(serde_json::Error),
    Proto(u16, String),
}

pub type PocketResult<T> = Result<T, PocketError>;

impl From<serde_json::Error> for PocketError {
    fn from(err: serde_json::Error) -> PocketError {
        PocketError::Json(err)
    }
}

impl From<IoError> for PocketError {
    fn from(err: IoError) -> PocketError {
        PocketError::Http(From::from(err))
    }
}

impl From<HttpError> for PocketError {
    fn from(err: HttpError) -> PocketError {
        PocketError::Http(err)
    }
}

impl Error for PocketError {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            PocketError::Http(ref e) => Some(e),
            PocketError::Json(ref e) => Some(e),
            PocketError::Proto(..) => None,
        }
    }
}

impl std::fmt::Display for PocketError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            PocketError::Http(ref e) => e.fmt(fmt),
            PocketError::Json(ref e) => e.fmt(fmt),
            PocketError::Proto(ref code, ref msg) => {
                fmt.write_str(&*format!("{} (code {})", msg, code))
            }
        }
    }
}

#[derive(Clone, Debug)]
struct XAccept(pub Mime);

impl std::ops::Deref for XAccept {
    type Target = Mime;
    fn deref(&self) -> &Mime {
        &self.0
    }
}

impl std::ops::DerefMut for XAccept {
    fn deref_mut(&mut self) -> &mut Mime {
        &mut self.0
    }
}

impl Header for XAccept {
    fn header_name() -> &'static str {
        "X-Accept"
    }

    fn parse_header(raw: &[Vec<u8>]) -> Result<XAccept, HttpError> {
        from_one_raw_str(raw).map(XAccept)
    }
}

impl HeaderFormat for XAccept {
    fn fmt_header(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, fmt)
    }
}

#[derive(Clone, Debug)]
struct XError(String);
#[derive(Clone, Debug)]
struct XErrorCode(u16);

impl Header for XError {
    fn header_name() -> &'static str {
        "X-Error"
    }

    fn parse_header(raw: &[Vec<u8>]) -> Result<XError, HttpError> {
        from_one_raw_str(raw).map(XError)
    }
}

impl HeaderFormat for XError {
    fn fmt_header(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, fmt)
    }
}

impl Header for XErrorCode {
    fn header_name() -> &'static str {
        "X-Error-Code"
    }

    fn parse_header(raw: &[Vec<u8>]) -> Result<XErrorCode, HttpError> {
        from_one_raw_str(raw).map(XErrorCode)
    }
}

impl HeaderFormat for XErrorCode {
    fn fmt_header(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, fmt)
    }
}

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

#[derive(Serialize)]
pub struct PocketAddRequest<'a> {
    #[serde(serialize_with = "borrow_url")]
    pub url: &'a url::Url,
    pub title: Option<&'a str>,
    #[serde(serialize_with = "to_comma_delimited_string")]
    pub tags: Option<&'a [&'a str]>,
    pub tweet_id: Option<&'a str>,
}

impl<'a> PocketAddRequest<'a> {
    pub fn new(url: &Url) -> PocketAddRequest {
        PocketAddRequest {
            url,
            title: None,
            tags: None,
            tweet_id: None
        }
    }

    pub fn title<'b>(&'b mut self, title: &'a str) -> &'b mut PocketAddRequest<'a> {
        self.title = Some(title);
        self
    }

    pub fn tags<'b>(&'b mut self, tags: &'a [&'a str]) -> &'b mut PocketAddRequest<'a> {
        self.tags = Some(tags);
        self
    }

    pub fn tweet_id<'b>(&'b mut self, tweet_id: &'a str) -> &'b mut PocketAddRequest<'a> {
        self.tweet_id = Some(tweet_id);
        self
    }
}

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

#[derive(Deserialize, Debug, PartialEq)]
pub struct PocketAddedItem {
    #[serde(deserialize_with = "from_str")]
    pub item_id: u64,

    #[serde(with = "url_serde")]
    pub normal_url: Url,

    #[serde(deserialize_with = "from_str")]
    pub resolved_id: u64,

    #[serde(deserialize_with = "from_str")]
    pub extended_item_id: u64,

    #[serde(deserialize_with = "try_url_from_string")]
    pub resolved_url: Option<Url>,

    #[serde(deserialize_with = "from_str")]
    pub domain_id: u64,
    #[serde(deserialize_with = "from_str")]
    pub origin_domain_id: u64,

    #[serde(deserialize_with = "from_str")]
    pub response_code: u16,

    #[serde(deserialize_with = "option_mime_from_string")]
    pub mime_type: Option<Mime>,

    #[serde(deserialize_with = "from_str")]
    pub content_length: usize,

    pub encoding: String,
    #[serde(deserialize_with = "option_string_date_format")]
    pub date_resolved: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "option_string_date_format")]
    pub date_published: Option<DateTime<Utc>>,

    pub title: String,
    pub excerpt: String,

    #[serde(deserialize_with = "from_str")]
    pub word_count: usize,

    #[serde(deserialize_with = "bool_from_int_string")]
    pub innerdomain_redirect: bool,
    #[serde(deserialize_with = "bool_from_int_string")]
    pub login_required: bool,

    pub has_image: PocketItemHas,
    pub has_video: PocketItemHas,

    #[serde(deserialize_with = "bool_from_int_string")]
    pub is_index: bool,
    #[serde(deserialize_with = "bool_from_int_string")]
    pub is_article: bool,

    #[serde(deserialize_with = "bool_from_int_string")]
    pub used_fallback: bool,

    #[serde(default)]
    pub lang: Option<String>,

    #[serde(deserialize_with = "option_string_date_unix_timestamp_format")]
    pub time_first_parsed: Option<DateTime<Utc>>,
    #[serde(default, deserialize_with = "optional_vec_from_map")]
    pub authors: Option<Vec<ItemAuthor>>,
    #[serde(default, deserialize_with = "optional_vec_from_map")]
    pub images: Option<Vec<PocketImage>>,
    #[serde(default, deserialize_with = "optional_vec_from_map")]
    pub videos: Option<Vec<ItemVideo>>,

    #[serde(default, deserialize_with = "try_url_from_string")]
    pub resolved_normal_url: Option<Url>,

    #[serde(with = "url_serde")]
    pub given_url: Url,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct PocketAddResponse {
    pub item: PocketAddedItem,
    pub status: u16,
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

#[derive(Serialize)]
pub struct PocketSendRequest<'a> {
    pub actions: &'a [&'a PocketSendAction],
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PocketSendAction {
    Add {
        #[serde(serialize_with = "optional_to_string")]
        item_id: Option<u64>,
        ref_id: Option<String>,
        tags: Option<String>,
        #[serde(serialize_with = "optional_to_string")]
        time: Option<u64>,
        title: Option<String>,
        #[serde(with = "url_serde")]
        url: Option<Url>,
    },
    Archive {
        #[serde(serialize_with = "to_string")]
        item_id: u64,
        #[serde(serialize_with = "optional_to_string")]
        time: Option<u64>,
    },
    Readd {
        #[serde(serialize_with = "to_string")]
        item_id: u64,
        #[serde(serialize_with = "optional_to_string")]
        time: Option<u64>,
    },
    Favorite {
        #[serde(serialize_with = "to_string")]
        item_id: u64,
        #[serde(serialize_with = "optional_to_string")]
        time: Option<u64>,
    },
    Unfavorite {
        #[serde(serialize_with = "to_string")]
        item_id: u64,
        #[serde(serialize_with = "optional_to_string")]
        time: Option<u64>,
    },
    Delete {
        #[serde(serialize_with = "to_string")]
        item_id: u64,
        #[serde(serialize_with = "optional_to_string")]
        time: Option<u64>,
    },
    TagsAdd {
        #[serde(serialize_with = "to_string")]
        item_id: u64,
        tags: String,
        #[serde(serialize_with = "optional_to_string")]
        time: Option<u64>,
    },
    TagsRemove {
        #[serde(serialize_with = "to_string")]
        item_id: u64,
        tags: String,
        #[serde(serialize_with = "optional_to_string")]
        time: Option<u64>,
    },
    TagsReplace {
        #[serde(serialize_with = "to_string")]
        item_id: u64,
        tags: String,
        #[serde(serialize_with = "optional_to_string")]
        time: Option<u64>,
    },
    TagsClear {
        #[serde(serialize_with = "to_string")]
        item_id: u64,
        #[serde(serialize_with = "optional_to_string")]
        time: Option<u64>,
    },
    TagRename {
        old_tag: String,
        new_tag: String,
        #[serde(serialize_with = "optional_to_string")]
        time: Option<u64>,
    },
    TagDelete {
        tag: String,
        #[serde(serialize_with = "optional_to_string")]
        time: Option<u64>,
    },
}

#[derive(Deserialize, Debug)]
pub struct PocketSendResponse {
    pub status: u16,
    pub action_results: Vec<bool>, // TODO - action_errors []
}

struct PocketClient {
    client: Client,
}

impl PocketClient {
    fn new() -> PocketClient {
        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);

        PocketClient {
            client: Client::with_connector(connector),
        }
    }

    fn get<T: IntoUrl, Resp: DeserializeOwned>(&self, url: T) -> PocketResult<Resp> {
        let request = self.client.get(url);
        self.request::<T, Resp>(request)
    }

    fn post<T: IntoUrl, B: Serialize, Resp: DeserializeOwned>(
        &self,
        url: T,
        body: &B,
    ) -> PocketResult<Resp> {
        let app_json: Mime = "application/json".parse().unwrap();
        let body = serde_json::to_string(body)?;
        let request = self
            .client
            .post(url)
            .body(&body)
            .header(ContentType(app_json.clone()))
            .header(XAccept(app_json));

        self.request::<T, Resp>(request)
    }

    fn request<T: IntoUrl, Resp: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> PocketResult<Resp> {
        request
            .send()
            .map_err(From::from)
            .and_then(|mut r| match r.headers.get::<XErrorCode>().map(|v| v.0) {
                None => {
                    let mut out = String::new();
                    r.read_to_string(&mut out).map_err(From::from).map(|_| out)
                }
                Some(code) => Err(PocketError::Proto(
                    code,
                    r.headers
                        .get::<XError>()
                        .map(|v| &*v.0)
                        .unwrap_or("unknown protocol error")
                        .to_string(),
                )),
            })
            .and_then(|s| serde_json::from_str(&*s).map_err(From::from))
    }
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
            client: PocketClient::new()
        }
    }

    pub fn request(&self, state: Option<&str>) -> PocketResult<String> {
        let body = &PocketOAuthRequest {
            consumer_key: &self.consumer_key,
            redirect_uri: &self.redirect_uri,
            state,
        };

        self.client
            .post("https://getpocket.com/v3/oauth/request", &body)
            .and_then(|r: PocketOAuthResponse| {
                PocketAuthentication::verify_state(state,r.state.as_deref())
                    .map(|()| r.code)
            })
    }

    fn verify_state(request_state: Option<&str>, response_state: Option<&str>) -> PocketResult<()> {
        match (request_state, response_state) {
            (Some(s1), Some(s2)) if s1 == s2 => Ok(()),
            (None, None) => Ok(()),
            _ => Err(PocketError::Proto(0, "State does not match".to_string()))
        }
    }

    pub fn authorize_url(&self, code: &str) -> Url {
        let params = vec![("request_token", code), ("redirect_uri", &self.redirect_uri)];
        let mut url = Url::parse("https://getpocket.com/auth/authorize").unwrap();
        url.query_pairs_mut().extend_pairs(params.into_iter());
        url
    }

    pub fn authorize(&self, code: &str, state: Option<&str>) -> PocketResult<PocketUser> {
        let body = &PocketAuthorizeRequest {
            consumer_key: &self.consumer_key,
            code,
        };

        self.client
            .post("https://getpocket.com/v3/oauth/authorize", &body)
            .and_then(|r: PocketAuthorizeResponse| {
                PocketAuthentication::verify_state(state, r.state.as_deref())
                    .map(|()| PocketUser {
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

impl PocketUser {
    pub fn pocket(self) -> Pocket {
        Pocket::new(&self.consumer_key, &self.access_token)
    }
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

    pub fn add(&self, request: &PocketAddRequest) -> PocketResult<PocketAddedItem> {
        let body = &PocketUserRequest {
            consumer_key: &*self.consumer_key,
            access_token: &*self.access_token,
            request,
        };

        self.client
            .post("https://getpocket.com/v3/add", &body)
            .map(|v: PocketAddResponse| v.item)
    }

    pub fn get(&self, request: &PocketGetRequest) -> PocketResult<Vec<PocketItem>> {
        let body = &PocketUserRequest {
            consumer_key: &*self.consumer_key,
            access_token: &*self.access_token,
            request,
        };

        self.client
            .post("https://getpocket.com/v3/get", &body)
            .map(|v: PocketGetResponse| v.list)
    }

    pub fn send(&self, request: &PocketSendRequest) -> PocketResult<PocketSendResponse> {
        let data = serde_json::to_string(request.actions)?;
        let params = &[
            ("consumer_key", &*self.consumer_key),
            ("access_token", &*self.access_token),
            ("actions", &data),
        ];

        let mut url = "https://getpocket.com/v3/send".into_url().unwrap();
        url.query_pairs_mut().extend_pairs(params.iter());

        self.client.get(url)
    }

    #[inline]
    pub fn push<T: IntoUrl>(&self, url: T) -> PocketResult<PocketAddedItem> {
        self.add(&PocketAddRequest::new(&url.into_url().unwrap()))
    }

    pub fn filter(&self) -> PocketGetRequest {
        PocketGetRequest::new()
    }
}

fn option_from_str<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let result: Result<T, D::Error> = from_str(deserializer);
    Ok(result.ok())
}

// https://github.com/serde-rs/json/issues/317
fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}

fn optional_to_string<T, S>(x: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: ToString,
    S: Serializer,
{
    match x {
        Some(ref value) => to_string(value, serializer),
        None => serializer.serialize_none(),
    }
}

fn to_string<T, S>(x: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: ToString,
    S: Serializer,
{
    serializer.serialize_str(&x.to_string())
}

fn to_comma_delimited_string<S>(x: &Option<&[&str]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    match x {
        Some(value) => {
            serializer.serialize_str(&value.join(","))
        },
        None => serializer.serialize_none(),
    }
}

fn try_url_from_string<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
    where
        D: Deserializer<'de>,
{
    let o: Option<String> = Option::deserialize(deserializer)?;
    Ok(o.and_then(|s| Url::parse(&s).ok()))
}

fn optional_vec_from_map<'de, T, D>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    T: DeserializeOwned + Clone + std::fmt::Debug,
    D: Deserializer<'de>,
{
    let o: Option<Value> = Option::deserialize(deserializer)?;
    match o {
        Some(v) => json_value_to_vec::<T, D>(v).map(Some),
        None => Ok(None),
    }
}

fn vec_from_map<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: DeserializeOwned + Clone + std::fmt::Debug,
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    json_value_to_vec::<T, D>(value)
}

fn json_value_to_vec<'de, T, D>(value: Value) -> Result<Vec<T>, D::Error>
where
    T: DeserializeOwned + Clone + std::fmt::Debug,
    D: Deserializer<'de>,
{
    match value {
        a @ Value::Array(..) => {
            serde_json::from_value::<Vec<T>>(a).map_err(serde::de::Error::custom)
        }
        o @ Value::Object(..) => serde_json::from_value::<BTreeMap<String, T>>(o)
            .map(map_to_vec)
            .map_err(serde::de::Error::custom),
        other => Err(serde::de::Error::invalid_value(
            Unexpected::Other(format!("{:?}", other).as_str()),
            &"object or array",
        )),
    }
}

fn map_to_vec<T>(map: BTreeMap<String, T>) -> Vec<T> {
    map.into_iter().map(|(_, v)| v).collect::<Vec<_>>()
}

// https://github.com/serde-rs/serde/issues/1344
fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(serde::de::Error::invalid_value(
            Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

fn bool_from_int_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer)?.as_str() {
        "0" => Ok(false),
        "1" => Ok(true),
        other => Err(serde::de::Error::invalid_value(
            Unexpected::Str(other),
            &"zero or one",
        )),
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn optional_bool_to_int<S>(x: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(ref value) => bool_to_int(value, serializer),
        None => serializer.serialize_none(),
    }
}

fn optional_datetime_to_int<S>(x: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(ref value) => string_date_unix_timestamp_format::serialize(value, serializer),
        None => serializer.serialize_none(),
    }
}

fn untagged_to_str<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str("_untagged_")
}

fn option_mime_from_string<'de, D>(deserializer: D) -> Result<Option<Mime>, D::Error>
    where
        D: Deserializer<'de>,
{
    Option::deserialize(deserializer)
        .and_then(|o: Option<String>| {
            match o.as_ref().map(|s| s.as_str()) {
                Some("") | None => Ok(None),
                Some(str) => str
                    .parse::<Mime>()
                    .map(Some)
                    .map_err(|other| serde::de::Error::invalid_value(
                        Unexpected::Other(format!("{:?}", other).as_str()),
                        &"valid mime type",
                    ))
            }
        })
}

fn int_date_unix_timestamp_format<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let unix_timestamp = i64::deserialize(deserializer)?;
    Ok(Utc.timestamp(unix_timestamp, 0))
}

fn option_string_date_unix_timestamp_format<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::deserialize(deserializer)
        .and_then(|o: Option<String>| {
            match o.as_ref().map(|s| s.as_str()) {
                Some("0") | None => Ok(None),
                Some(str) => str
                    .parse::<i64>()
                    .map(|i| Some(Utc.timestamp(i, 0)))
                    .map_err(serde::de::Error::custom)
            }
        })
}

const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

fn option_string_date_format<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
{
        match String::deserialize(deserializer)?.as_str() {
            "0000-00-00 00:00:00" => Ok(None),
            str => Utc.datetime_from_str(str, FORMAT)
                .map_err(serde::de::Error::custom)
                .map(Option::Some)
        }
}

// inspired by https://serde.rs/custom-date-format.html
mod string_date_unix_timestamp_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&date.timestamp().to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<i64>()
            .map(|i| Utc.timestamp(i, 0))
            .map_err(serde::de::Error::custom)
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn bool_to_int<S>(x: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let output = match x {
        true => "1",
        false => "0",
    };
    serializer.serialize_str(output)
}

fn borrow_url<S>(x: &Url, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    serializer.serialize_str(x.as_str())
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Serialize;

    // Auth
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

    fn remove_whitespace(s: &str) -> String {
        s.replace(|c: char| c.is_whitespace(), "")
    }

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
            src: "http://localhost".into_url().ok(),
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

    // PocketAddRequest
    #[test]
    fn test_serialize_add_request() {
        let tags = &["tags"];
        let request = &PocketAddRequest {
            url: &"http://localhost".into_url().unwrap(),
            title: Some("title"),
            tags: Some(tags),
            tweet_id: Some("tweet_id"),
        };

        let actual = serde_json::to_string(request).unwrap();

        let expected = remove_whitespace(&format!(
            r#"
                    {{
                        "url": "{url}",
                        "title": "{title}",
                        "tags": "{tags}",
                        "tweet_id": "{tweet_id}"
                    }}
               "#,
            url = request.url,
            title = request.title.unwrap(),
            tags = request.tags.unwrap().join(","),
            tweet_id = request.tweet_id.unwrap(),
        ));

        assert_eq!(actual, expected);
    }

    // PocketAddedItem

    // PocketAddResponse
    #[test]
    fn test_deserialize_add_response_resolved_url() {
        let expected = PocketAddResponse {
            item: PocketAddedItem {
                item_id: 2763821,
                normal_url: "http://example.com".into_url().unwrap(),
                resolved_id: 2763821,
                extended_item_id: 2763821,
                resolved_url: "https://example.com".into_url().ok(),
                domain_id: 85964,
                origin_domain_id: 51347065,
                response_code: 200,
                mime_type: "text/html".parse().ok(),
                content_length: 648,
                encoding: "utf-8".to_string(),
                date_resolved: Utc.datetime_from_str("2020-03-03 12:20:37", FORMAT).ok(),
                date_published: None,
                title: "Example Domain".to_string(),
                excerpt: "This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission. More information...".to_string(),
                word_count: 28,
                innerdomain_redirect: true,
                login_required: false,
                has_image: PocketItemHas::No,
                has_video: PocketItemHas::No,
                is_index: true,
                is_article: false,
                used_fallback: true,
                lang: Some("".to_string()),
                time_first_parsed: None,
                authors: Some(vec![]),
                images: Some(vec![]),
                videos: Some(vec![]),
                resolved_normal_url: "http://example.com".into_url().ok(),
                given_url: "https://example.com".into_url().unwrap(),
            },
            status: 1,
        };
        let response = r#"
            {
                "item": {
                    "item_id": "2763821",
                    "normal_url": "http://example.com",
                    "resolved_id": "2763821",
                    "extended_item_id": "2763821",
                    "resolved_url": "https://example.com",
                    "domain_id": "85964",
                    "origin_domain_id": "51347065",
                    "response_code": "200",
                    "mime_type": "text/html",
                    "content_length": "648",
                    "encoding": "utf-8",
                    "date_resolved": "2020-03-03 12:20:37",
                    "date_published": "0000-00-00 00:00:00",
                    "title": "Example Domain",
                    "excerpt": "This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission. More information...",
                    "word_count": "28",
                    "innerdomain_redirect": "1",
                    "login_required": "0",
                    "has_image": "0",
                    "has_video": "0",
                    "is_index": "1",
                    "is_article": "0",
                    "used_fallback": "1",
                    "lang": "",
                    "time_first_parsed": "0",
                    "authors": [],
                    "images": [],
                    "videos": [],
                    "resolved_normal_url": "http://example.com",
                    "given_url": "https://example.com"
                },
                "status": 1
            }
       "#;

        let actual: PocketAddResponse = serde_json::from_str(&response).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_deserialize_add_response_unresolved_url() {
        let expected = PocketAddResponse {
            item: PocketAddedItem {
                item_id: 1933886793,
                normal_url: "http://dc7ad3b2-942e-41c5-9154-a1b545752102.com".into_url().unwrap(),
                resolved_id: 0,
                extended_item_id: 0,
                resolved_url: None,
                domain_id: 0,
                origin_domain_id: 0,
                response_code: 0,
                mime_type: None,
                content_length: 0,
                encoding: "".to_string(),
                date_resolved: None,
                date_published: None,
                title: "".to_string(),
                excerpt: "".to_string(),
                word_count: 0,
                innerdomain_redirect: false,
                login_required: false,
                has_image: PocketItemHas::No,
                has_video: PocketItemHas::No,
                is_index: false,
                is_article: false,
                used_fallback: false,
                lang: None,
                time_first_parsed: None,
                authors: None,
                images: None,
                videos: None,
                resolved_normal_url: None,
                given_url: "https://dc7ad3b2-942e-41c5-9154-a1b545752102.com".into_url().unwrap(),
            },
            status: 1,
        };
        let response = r#"
            {
                "item": {
                    "item_id": "1933886793",
                    "normal_url": "http://dc7ad3b2-942e-41c5-9154-a1b545752102.com",
                    "resolved_id": "0",
                    "extended_item_id": "0",
                    "resolved_url": "",
                    "domain_id": "0",
                    "origin_domain_id": "0",
                    "response_code": "0",
                    "mime_type": "",
                    "content_length": "0",
                    "encoding": "",
                    "date_resolved": "0000-00-00 00:00:00",
                    "date_published": "0000-00-00 00:00:00",
                    "title": "",
                    "excerpt": "",
                    "word_count": "0",
                    "innerdomain_redirect": "0",
                    "login_required": "0",
                    "has_image": "0",
                    "has_video": "0",
                    "is_index": "0",
                    "is_article": "0",
                    "used_fallback": "0",
                    "lang": null,
                    "time_first_parsed": null,
                    "given_url": "https://dc7ad3b2-942e-41c5-9154-a1b545752102.com"
                },
                "status": 1
            }
       "#;

        let actual: PocketAddResponse = serde_json::from_str(&response).unwrap();

        assert_eq!(actual, expected);
    }

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
