use serde::{Deserialize, Serialize};
use crate::serialization::*;
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

#[derive(Deserialize, Debug)]
pub struct PocketSendResponse {
    pub status: u16,
    pub action_results: Vec<bool>, // TODO - action_errors []
}