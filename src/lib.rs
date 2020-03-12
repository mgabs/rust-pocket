extern crate hyper;
extern crate url;
extern crate mime;
extern crate chrono;
extern crate hyper_native_tls;

#[macro_use] extern crate serde_derive;
extern crate serde;

use hyper_native_tls::NativeTlsClient;
use hyper::header::{Header, HeaderFormat, ContentType};
use hyper::client::{Client, IntoUrl, RequestBuilder};
use hyper::header::parsing::from_one_raw_str;
use hyper::error::Error as HttpError;
use url::Url;
use mime::Mime;
use std::error::Error;
use std::convert::From;
use std::io::Error as IoError;
use std::io::Read;
use std::result::Result;
use serde::{Deserialize, Deserializer, Serializer};
use serde::de::{DeserializeOwned, Unexpected};
use std::str::FromStr;
use std::fmt::Display;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use hyper::net::HttpsConnector;

#[derive(Debug)]
pub enum PocketError {
    Http(HttpError),
    Json(serde_json::Error),
    Proto(u16, String)
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
    fn description(&self) -> &str {
        match *self {
            PocketError::Http(ref e) => e.description(),
            PocketError::Json(ref e) => e.description(),
            PocketError::Proto(..) => "protocol error"
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            PocketError::Http(ref e) => Some(e),
            PocketError::Json(ref e) => Some(e),
            PocketError::Proto(..) => None
        }
    }
}

impl std::fmt::Display for PocketError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            PocketError::Http(ref e) => e.fmt(fmt),
            PocketError::Json(ref e) => e.fmt(fmt),
            PocketError::Proto(ref code, ref msg) => fmt.write_str(&*format!("{} (code {})", msg, code))
        }
    }
}

#[derive(Clone, Debug)]
struct XAccept(pub Mime);

impl std::ops::Deref for XAccept {
    type Target = Mime;
    fn deref<'a>(&'a self) -> &'a Mime {
        &self.0
    }
}

impl std::ops::DerefMut for XAccept {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Mime {
        &mut self.0
    }
}

impl Header for XAccept {
    fn header_name() -> &'static str {
        "X-Accept"
    }

    fn parse_header(raw: &[Vec<u8>]) -> Result<XAccept, HttpError> {
        from_one_raw_str(raw).map(|mime| XAccept(mime))
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
        from_one_raw_str(raw).map(|error| XError(error))
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
        from_one_raw_str(raw).map(|code| XErrorCode(code))
    }
}

impl HeaderFormat for XErrorCode {
    fn fmt_header(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, fmt)
    }
}

pub struct Pocket {
    consumer_key: String,
    access_token: Option<String>,
    code: Option<String>,
    client: Client
}

#[derive(Serialize)]
pub struct PocketOAuthRequest<'a> {
    consumer_key: &'a str,
    redirect_uri: &'a str,
    state: Option<&'a str>
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct PocketOAuthResponse {
    code: String,
    state: Option<String>
}

#[derive(Serialize)]
pub struct PocketAuthorizeRequest<'a> {
    consumer_key: &'a str,
    code: &'a str
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct PocketAuthorizeResponse {
    access_token: String,
    username: String
}

#[derive(Serialize)]
struct PocketAddRequest<'a> {
    #[serde(with = "url_serde")]
    pub url: url::Url, // TODO - borrow
    pub title: Option<&'a str>,
    pub tags: Option<&'a str>, // TODO - make vec or array
    pub tweet_id: Option<&'a str>
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct ItemImage {
    #[serde(deserialize_with = "from_str")]
    pub item_id: u64,
    #[serde(deserialize_with = "from_str")]
    pub image_id: u64,
    #[serde(with = "url_serde")]
    pub src: Url,
    #[serde(deserialize_with = "from_str")]
    pub width: u16,
    #[serde(deserialize_with = "from_str")]
    pub height: u16,
    pub credit: String,
    pub caption: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct DomainMetaData {
    pub name: String,
    pub logo: String,
    pub greyscale_logo: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct ItemVideo {
    #[serde(deserialize_with = "from_str")]
    pub item_id: u64,
    #[serde(deserialize_with = "from_str")]
    pub video_id: u64,
    #[serde(with = "url_serde")]
    pub src: Url,
    #[serde(deserialize_with = "from_str")]
    pub width: u16,
    #[serde(deserialize_with = "from_str")]
    pub height: u16,
    #[serde(deserialize_with = "option_from_str")]
    pub length: Option<usize>,
    pub vid: String,
    #[serde(rename="type")]
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
    #[serde(rename="0")]
    No,
    #[serde(rename="1")]
    Yes,
    #[serde(rename="2")]
    Is
}

// TODO - compare with PocketItem
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

    #[serde(with = "url_serde")]
    pub resolved_url: Url,

    #[serde(deserialize_with = "from_str")]
    pub domain_id: u64,
    #[serde(deserialize_with = "from_str")]
    pub origin_domain_id: u64,

    #[serde(deserialize_with = "from_str")]
    pub response_code: u16,

    pub mime_type: String, // must be Option<Mime>

    #[serde(deserialize_with = "from_str")]
    pub content_length: usize,

    pub encoding: String,
    pub date_resolved: String, // TODO - 2020-03-02 14:51:51
    pub date_published: String, // TODO - 0000-00-00 00:00:00

    pub title: String,
    pub excerpt: String,

    #[serde(deserialize_with = "from_str")]
    pub word_count: usize,

    // TODO - innerdomain_redirect 1

    #[serde(deserialize_with = "bool_from_int")]
    pub login_required: bool,

    pub has_image: PocketItemHas,
    pub has_video: PocketItemHas,

    #[serde(deserialize_with = "bool_from_int")]
    pub is_index: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub is_article: bool,

    #[serde(deserialize_with = "bool_from_int")]
    pub used_fallback: bool,

    pub lang: String,

    // TODO - time_first_parsed 0

    pub authors: Vec<ItemAuthor>,
    pub images: Vec<ItemImage>,

    pub videos: Vec<ItemVideo>,

    #[serde(with = "url_serde")]
    pub given_url: Url,
}

#[derive(Deserialize, Debug)]
pub struct PocketAddResponse {
    pub item: PocketAddedItem,
    pub status: u16
}

#[derive(Serialize)]
pub struct PocketUserRequest<'a, T> {
    consumer_key: &'a str,
    access_token: &'a str,

    #[serde(flatten)]
    request: T,
}

#[derive(Serialize)]
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
    offset: Option<usize>
}

impl<'a> PocketGetRequest<'a> {
    pub fn new() -> PocketGetRequest<'a> {
        PocketGetRequest {
            search: None,
            domain: None,
            tag: None,
            state: None,
            content_type: None,
            detail_type: None,
            favorite: None,
            since: None,
            sort: None,
            count: None,
            offset: None
        }
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

    pub fn content_type<'b>(&'b mut self, content_type: PocketGetType) -> &'b mut PocketGetRequest<'a> {
        self.content_type = Some(content_type);
        self
    }

    pub fn detail_type<'b>(&'b mut self, detail_type: PocketGetDetail) -> &'b mut PocketGetRequest<'a> {
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
#[serde(rename_all="lowercase")]
pub enum PocketGetDetail {
    Simple,
    Complete
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all="lowercase")]
pub enum PocketGetSort {
    Newest,
    Oldest,
    Title,
    Site
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all="lowercase")]
pub enum PocketGetState {
    Unread,
    Archive,
    All
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum PocketGetTag<'a> {
    #[serde(serialize_with="untagged_to_str")]
    Untagged,
    Tagged(&'a str)
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all="lowercase")]
pub enum PocketGetType {
    Article,
    Video,
    Image
}

#[derive(Deserialize, Debug)]
struct PocketSearchMeta {
    search_type: String
}

#[derive(Deserialize, Debug)]
struct PocketGetResponse {
    #[serde(deserialize_with = "vec_from_map")]
    list: Vec<PocketItem>,
    status: u16,
    complete: u16, // TODO - map to bool
    error: Option<String>,
    search_meta: PocketSearchMeta,
    // TODO - deserialize option datetime utc
    //since: Option<DateTime<Utc>>
}

#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum PocketItemStatus {
    #[serde(rename="0")]
    Normal,
    #[serde(rename="1")]
    Archived,
    #[serde(rename="2")]
    Deleted
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct PocketItem {
    #[serde(deserialize_with = "from_str")]
    pub item_id: u64,

    #[serde(with = "url_serde")]
    pub given_url: Url,
    pub given_title: String,

    #[serde(deserialize_with = "from_str")]
    pub word_count: usize,
    pub excerpt: String,

    #[serde(with="date_unix_timestamp_format")]
    pub time_added: DateTime<Utc>,
    #[serde(with="date_unix_timestamp_format")]
    pub time_read: DateTime<Utc>,
    #[serde(with="date_unix_timestamp_format")]
    pub time_updated: DateTime<Utc>, // TODO - change to None if zero?
    #[serde(with="date_unix_timestamp_format")]
    pub time_favorited: DateTime<Utc>,

    #[serde(deserialize_with="bool_from_int")]
    pub favorite: bool,

    #[serde(deserialize_with="bool_from_int")]
    pub is_index: bool,
    #[serde(deserialize_with="bool_from_int")]
    pub is_article: bool,
    pub has_image: PocketItemHas,
    pub has_video: PocketItemHas,

    #[serde(deserialize_with = "from_str")]
    pub resolved_id: u64,
    pub resolved_title: String,
    pub resolved_url: String, // not always a valid url

    pub sort_id: u64,

    pub status: PocketItemStatus,
    #[serde(default, deserialize_with = "optional_vec_from_map")]
    pub images: Option<Vec<ItemImage>>,
    #[serde(default, deserialize_with = "optional_vec_from_map")]
    pub videos: Option<Vec<ItemVideo>>,
    #[serde(default, deserialize_with = "optional_vec_from_map")]
    pub authors: Option<Vec<ItemAuthor>>,
    pub lang: String,
    pub time_to_read: Option<u64>,
    pub domain_metadata: Option<DomainMetaData>,
    pub listen_duration_estimate: Option<u64>,

    // pub image: Option<ItemImage>, // TODO - does not have image_id
    // TODO - amp_url
    // TODO - top_image_url
}

#[derive(Serialize)]
pub struct PocketSendRequest<'a> {
    pub actions: &'a [&'a PocketSendAction]
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PocketSendAction {
    Add {
        #[serde(serialize_with="optional_to_string")]
        item_id: Option<u64>,
        ref_id: Option<String>,
        tags: Option<String>,
        #[serde(serialize_with="optional_to_string")]
        time: Option<u64>,
        title: Option<String>,
        #[serde(with = "url_serde")]
        url: Option<Url>
    },
    Archive {
        #[serde(serialize_with="to_string")]
        item_id: u64,
        #[serde(serialize_with="optional_to_string")]
        time: Option<u64>
    },
    Readd {
        #[serde(serialize_with="to_string")]
        item_id: u64,
        #[serde(serialize_with="optional_to_string")]
        time: Option<u64>
    },
    Favorite {
        #[serde(serialize_with="to_string")]
        item_id: u64,
        #[serde(serialize_with="optional_to_string")]
        time: Option<u64>
    },
    Unfavorite {
        #[serde(serialize_with="to_string")]
        item_id: u64,
        #[serde(serialize_with="optional_to_string")]
        time: Option<u64>
    },
    Delete {
        #[serde(serialize_with="to_string")]
        item_id: u64,
        #[serde(serialize_with="optional_to_string")]
        time: Option<u64>
    } ,
    TagsAdd {
        #[serde(serialize_with="to_string")]
        item_id: u64,
        tags: String,
        #[serde(serialize_with="optional_to_string")]
        time: Option<u64>
    },
    TagsRemove {
        #[serde(serialize_with="to_string")]
        item_id: u64,
        tags: String,
        #[serde(serialize_with="optional_to_string")]
        time: Option<u64>
    },
    TagsReplace {
        #[serde(serialize_with="to_string")]
        item_id: u64,
        tags: String,
        #[serde(serialize_with="optional_to_string")]
        time: Option<u64>
    },
    TagsClear {
        #[serde(serialize_with="to_string")]
        item_id: u64,
        #[serde(serialize_with="optional_to_string")]
        time: Option<u64>
    },
    TagRename {
        old_tag: String,
        new_tag: String,
        #[serde(serialize_with="optional_to_string")]
        time: Option<u64>
    },
    TagDelete {
        tag: String,
        #[serde(serialize_with="optional_to_string")]
        time: Option<u64>
    },
}

#[derive(Deserialize, Debug)]
pub struct PocketSendResponse {
    pub status: u16,
    pub action_results: Vec<bool>
    // TODO - action_errors []
}

impl Pocket {
    pub fn new(consumer_key: &str, access_token: Option<&str>) -> Pocket {
        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);

        Pocket {
            consumer_key: consumer_key.to_string(),
            access_token: access_token.map(|v| v.to_string()),
            code: None,
            client: Client::with_connector(connector)
        }
    }

    #[inline] pub fn access_token(&self) -> Option<&str> {
        self.access_token.as_ref().map(|v| &**v)
    }

    fn request<T: IntoUrl, Resp: DeserializeOwned>(&self, request: RequestBuilder) -> PocketResult<Resp> {
        request.send()
            .map_err(From::from)
            .and_then(|mut r| match r.headers.get::<XErrorCode>().map(|v| v.0) {
                None => {
                    let mut out = String::new();
                    r.read_to_string(&mut out).map_err(From::from).map(|_| out)
                },
                Some(code) => Err(PocketError::Proto(code, r.headers.get::<XError>().map(|v| &*v.0)
                    .unwrap_or("unknown protocol error").to_string())),
            })
            .and_then(|s| serde_json::from_str(&*s).map_err(From::from))
    }

    fn get_request<T: IntoUrl, Resp: DeserializeOwned>(&self, url: T) -> PocketResult<Resp> {
        self.request::<T, Resp>(self.client.get(url))
    }

    fn post_request<T: IntoUrl, Resp: DeserializeOwned>(&self, url: T, data: &str) -> PocketResult<Resp> {
        let app_json: Mime = "application/json".parse().unwrap();

        self.request::<T, Resp>(self.client
            .post(url)
            .body(data)
            .header(ContentType(app_json.clone()))
            .header(XAccept(app_json.clone()))
        )
    }

    pub fn get_auth_url(&mut self) -> PocketResult<Url> {
        let request = serde_json::to_string(&PocketOAuthRequest {
            consumer_key: &*self.consumer_key,
            redirect_uri: "rustapi:finishauth",
            state: None
        })?;

        self.post_request("https://getpocket.com/v3/oauth/request", &*request)
            .and_then(|r: PocketOAuthResponse| {
                let mut url = Url::parse("https://getpocket.com/auth/authorize").unwrap();
                url.query_pairs_mut().extend_pairs(vec![("request_token", &*r.code), ("redirect_uri", "rustapi:finishauth")].into_iter());
                self.code = Some(r.code);
                Ok(url)
            })
    }

    pub fn authorize(&mut self) -> PocketResult<String> {
        let request = serde_json::to_string(&PocketAuthorizeRequest {
            consumer_key: &*self.consumer_key,
            code: self.code.as_ref().map(|v| &**v).unwrap()
        })?;

        match self.post_request("https://getpocket.com/v3/oauth/authorize", &*request)
        {
            Ok(r @ PocketAuthorizeResponse {..}) => {
                self.access_token = Some(r.access_token);
                Ok(r.username)
            },
            Err(e) => Err(e)
        }
    }

    pub fn add<T: IntoUrl>(&self, url: T, title: Option<&str>, tags: Option<&str>, tweet_id: Option<&str>) -> PocketResult<PocketAddedItem> {
        let data = serde_json::to_string(&PocketUserRequest {
            consumer_key: &*self.consumer_key,
            access_token: &**self.access_token.as_ref().unwrap(),
            request: &PocketAddRequest {
                url: url.into_url().unwrap(),
                title: title.map(|v| v.clone()),
                tags: tags.map(|v| v.clone()),
                tweet_id: tweet_id.map(|v| v.clone())
            }
        })?;

        self.post_request("https://getpocket.com/v3/add", &data)
            .map(|v: PocketAddResponse| v.item)
    }

    pub fn get(&self, request: &PocketGetRequest) -> PocketResult<Vec<PocketItem>> {
        let data = serde_json::to_string(&PocketUserRequest {
            consumer_key: &*self.consumer_key,
            access_token: &**self.access_token.as_ref().unwrap(),
            request
        })?;

        self.post_request("https://getpocket.com/v3/get", &data)
            .map(|v: PocketGetResponse| v.list)
    }

    pub fn send(&self, request: &PocketSendRequest) -> PocketResult<PocketSendResponse> {
        let data = serde_json::to_string(request.actions)?;

        let params = &[
            ("consumer_key", &*self.consumer_key),
            ("access_token", &**self.access_token.as_ref().unwrap()),
            ("actions", &data)
        ];

        let mut url = "https://getpocket.com/v3/send".into_url().unwrap();
        url.query_pairs_mut().extend_pairs(params.into_iter());

        self.get_request(url)
    }

    #[inline] pub fn push<T: IntoUrl>(&mut self, url: T) -> PocketResult<PocketAddedItem> {
        self.add(url, None, None, None)
    }

    pub fn filter(&self) -> PocketGetRequest {
        PocketGetRequest::new()
    }
}

fn option_from_str<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where T: FromStr,
          T::Err:Display,
          D: Deserializer<'de>
{
    let result: Result<T, D::Error> = from_str(deserializer);
    result.map(Option::from)
}

// https://github.com/serde-rs/json/issues/317
fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err:Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}

fn optional_to_string<T, S>(x: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where T: ToString,
          S: Serializer
{
    match x {
        Some(ref value) => to_string(value, serializer),
        None => serializer.serialize_none()
    }
}

fn to_string<T, S>(x: &T, serializer: S) -> Result<S::Ok, S::Error>
    where T: ToString,
          S: Serializer
{
    serializer.serialize_str(&x.to_string())
}

fn optional_vec_from_map<'de, T, D>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
    where T: Deserialize<'de> + Clone + std::fmt::Debug,
          D: Deserializer<'de>
{
    let m: Option< HashMap<String, T>> = Option::deserialize(deserializer)?;
    Ok(m.map(map_to_vec))
}

fn vec_from_map<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where T: Deserialize<'de> + Clone + std::fmt::Debug,
          D: Deserializer<'de>
{
    let map: HashMap<String, T> = HashMap::deserialize(deserializer)?;
    let result = map_to_vec(map);

    Ok(result)
}

fn map_to_vec<T>(map: HashMap<String, T>) -> Vec<T>
    where T: Clone + std::fmt::Debug
{
    let mut result = vec![];
    let mut kvs = map.iter().collect::<Vec<_>>();
    // TODO - items have a sort id, images and such have and index - rethink this sort, possibly a better map type?
    kvs.sort_by(|&a, &b| a.0.cmp(b.0) );

    for (_, v) in kvs {
        result.push(v.clone());
    }

    result
}

// https://github.com/serde-rs/serde/issues/1344
fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
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

fn optional_bool_to_int<S>(x: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    match x {
        Some(ref value) => bool_to_int(value, serializer),
        None => serializer.serialize_none()
    }
}

fn optional_datetime_to_int<S>(x: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    match x {
        Some(ref value) => date_unix_timestamp_format::serialize(value, serializer),
        None => serializer.serialize_none()
    }
}

fn untagged_to_str<S>(serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    serializer.serialize_str("_untagged_")
}

// inspired by https://serde.rs/custom-date-format.html
mod date_unix_timestamp_format {
    use chrono::{DateTime, Utc, TimeZone};
    use serde::{self, Deserialize, Serializer, Deserializer};

    pub fn serialize<S>(
        date: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(&date.timestamp().to_string())
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<i64>()
            .map(|i| Utc.timestamp(i, 0))
            .map_err(serde::de::Error::custom)
    }
}

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

        let expected = remove_whitespace(&format!(r#"
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
        let response = remove_whitespace(&format!(r#"
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

        let expected = remove_whitespace(&format!(r#"
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
        };
        let response = remove_whitespace(&format!(r#"
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

    fn remove_whitespace(s: &String) -> String {
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
            offset: Some(2)
        };

        let actual = serde_json::to_string(request).unwrap();

        let expected = remove_whitespace(&format!(r#"
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
        serde_json::to_value(value).unwrap()
            .as_str().unwrap()
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
            image_id: 2,
            src: "http://localhost".into_url().unwrap(),
            width: 3,
            height: 4,
            caption: "caption".to_string(),
            credit: "credit".to_string(),
        };
        let response = remove_whitespace(&format!(r#"
                    {{
                        "item_id": "{item_id}",
                        "image_id": "{image_id}",
                        "src": "{src}",
                        "width": "{width}",
                        "height": "{height}",
                        "caption": "{caption}",
                        "credit": "{credit}"
                    }}
               "#,
              item_id = expected.item_id,
              image_id = expected.image_id,
              src = expected.src,
              width = expected.width,
              height = expected.height,
              caption = expected.caption,
              credit = expected.credit,
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
        let request = &PocketAddRequest {
            url: "http://localhost".into_url().unwrap(),
            title: Some("title"),
            tags: Some("tags"),
            tweet_id: Some("tweet_id"),
        };

        let actual = serde_json::to_string(request).unwrap();

        let expected = remove_whitespace(&format!(r#"
                    {{
                        "url": "{url}",
                        "title": "{title}",
                        "tags": "{tags}",
                        "tweet_id": "{tweet_id}"
                    }}
               "#,
              url = request.url,
              title = request.title.unwrap(),
              tags = request.tags.unwrap(),
              tweet_id = request.tweet_id.unwrap(),
        ));

        assert_eq!(actual, expected);
    }

    // PocketAddedItem
    // PocketAddResponse
    // PocketGetResponse
    // PocketItemStatus
    // PocketItem
}
