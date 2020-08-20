use crate::serialization::*;
use crate::{ItemAuthor, ItemVideo, PocketImage, PocketItemHas};
use chrono::{DateTime, Utc};
use mime::Mime;
use serde::{Deserialize, Serialize};
use url::Url;

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
            tweet_id: None,
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::remove_whitespace;
    use chrono::TimeZone;

    // PocketAddRequest
    #[test]
    fn test_serialize_add_request() {
        let tags = &["tags"];
        let request = &PocketAddRequest {
            url: &Url::parse("http://localhost").unwrap(),
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
                  normal_url: Url::parse("http://example.com").unwrap(),
                  resolved_id: 2763821,
                  extended_item_id: 2763821,
                  resolved_url: Url::parse("https://example.com").ok(),
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
                  resolved_normal_url: Url::parse("http://example.com").ok(),
                  given_url: Url::parse("https://example.com").unwrap(),
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
                normal_url: Url::parse("http://dc7ad3b2-942e-41c5-9154-a1b545752102.com").unwrap(),
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
                given_url: Url::parse("https://dc7ad3b2-942e-41c5-9154-a1b545752102.com").unwrap(),
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
}
