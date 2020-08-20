use crate::{add::PocketAddedItem, serialization::*};
use serde::{Deserialize, Serialize};
use url::Url;

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

#[derive(Deserialize, Debug, PartialEq)]
pub struct PocketSendResponse {
    pub status: u16,
    pub action_results: Vec<SendActionResult>,
    pub action_errors: Vec<Option<SendActionError>>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum SendActionResult {
    #[serde(deserialize_with = "true_to_unit_variant")]
    Success,
    #[serde(deserialize_with = "false_to_unit_variant")]
    Failure,
    Add(Box<PocketAddedItem>),
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct SendActionError {
    code: u16,
    message: String,
    #[serde(rename = "type")]
    error_type: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::PocketItemHas;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_deserialize_send_response() {
        let expected = PocketSendResponse {
            status: 1,
            action_results: vec![
                SendActionResult::Success,
                SendActionResult::Failure,
            ],
            action_errors: vec![
                None,
                Some(SendActionError {
                    code: 422,
                    message: "Invalid/non-existent URL".to_string(),
                    error_type: "Unprocessable Entity".to_string(),
                }),
            ],
        };
        let response = r#"
            {
                "action_results":[
                    true,
                    false
                ],
                "action_errors":[
                    null,
                    {
                        "code": 422,
                        "message": "Invalid/non-existent URL",
                        "type": "Unprocessable Entity"
                    }
                ],
                "status":1
            }
        "#;

        let actual: PocketSendResponse = serde_json::from_str(&response).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_deserialize_send_response_add() {
        let expected = PocketSendResponse {
            status: 1,
            action_results: vec![
                SendActionResult::Add(
                    Box::new(PocketAddedItem {
                        item_id: 1502819,
                        normal_url: Url::parse("http://example.com").unwrap(),
                        resolved_id: 1502819,
                        extended_item_id: 1502819,
                        resolved_url: Url::parse("https://example.com").ok(),
                        domain_id: 85964,
                        origin_domain_id: 772,
                        response_code: 200,
                        mime_type: "text/html".parse().ok(),
                        content_length: 648,
                        encoding: "utf-8".to_string(),
                        date_resolved: Utc.datetime_from_str("2020-08-04 22:41:28", FORMAT).ok(),
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
                        given_url: Url::parse("https://example.com/").unwrap(),
                    })
                ),
            ],
            action_errors: vec![None],
        };
        let response = r#"
            {
                "action_results":[
                {
                    "item_id":"1502819",
                    "normal_url":"http://example.com",
                    "resolved_id":"1502819",
                    "extended_item_id":"1502819",
                    "resolved_url":"https:example.com/",
                    "domain_id":"85964",
                    "origin_domain_id":"772",
                    "response_code":"200",
                    "mime_type":"text/html",
                    "content_length":"648",
                    "encoding":"utf-8",
                    "date_resolved":"2020-08-04 22:41:28",
                    "date_published":"0000-00-00 00:00:00",
                    "title":"Example Domain",
                    "excerpt":"This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission. More information...",
                    "word_count":"28",
                    "innerdomain_redirect":"1",
                    "login_required":"0",
                    "has_image":"0",
                    "has_video":"0",
                    "is_index":"1",
                    "is_article":"0",
                    "used_fallback":"1",
                    "lang":"",
                    "time_first_parsed":"0",
                    "authors":[
            
                    ],
                    "images":[
            
                    ],
                    "videos":[
            
                    ],
                    "resolved_normal_url":"http://example.com",
                    "given_url":"https://example.com/"
                }
                ],
                "action_errors":[
                    null
                ],
                "status":1
            }
        "#;

        let actual: PocketSendResponse = serde_json::from_str(&response).unwrap();

        assert_eq!(actual, expected);
    }
}
