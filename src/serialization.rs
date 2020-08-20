use chrono::{DateTime, TimeZone, Utc};
use mime::Mime;
use serde::de::{DeserializeOwned, Unexpected};
use serde::{Deserialize, Deserializer, Serializer};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::result::Result;
use std::str::FromStr;
use url::Url;

pub fn option_from_str<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let result: Result<T, D::Error> = from_str(deserializer);
    Ok(result.ok())
}

// https://github.com/serde-rs/json/issues/317
pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}

pub fn optional_to_string<T, S>(x: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: ToString,
    S: Serializer,
{
    match x {
        Some(ref value) => to_string(value, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn to_string<T, S>(x: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: ToString,
    S: Serializer,
{
    serializer.serialize_str(&x.to_string())
}

pub fn to_comma_delimited_string<S>(x: &Option<&[&str]>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(value) => serializer.serialize_str(&value.join(",")),
        None => serializer.serialize_none(),
    }
}

pub fn try_url_from_string<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
where
    D: Deserializer<'de>,
{
    let o: Option<String> = Option::deserialize(deserializer)?;
    Ok(o.and_then(|s| Url::parse(&s).ok()))
}

pub fn optional_vec_from_map<'de, T, D>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
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

pub fn vec_from_map<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: DeserializeOwned + Clone + std::fmt::Debug,
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    json_value_to_vec::<T, D>(value)
}

pub fn json_value_to_vec<'de, T, D>(value: Value) -> Result<Vec<T>, D::Error>
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

pub fn map_to_vec<T>(map: BTreeMap<String, T>) -> Vec<T> {
    map.into_iter().map(|(_, v)| v).collect::<Vec<_>>()
}

// https://github.com/serde-rs/serde/issues/1344
pub fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
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

pub fn bool_from_int_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
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
pub fn optional_bool_to_int<S>(x: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(ref value) => bool_to_int(value, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn optional_datetime_to_int<S>(
    x: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(ref value) => string_date_unix_timestamp_format::serialize(value, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn untagged_to_str<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str("_untagged_")
}

pub fn option_mime_from_string<'de, D>(deserializer: D) -> Result<Option<Mime>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::deserialize(deserializer).and_then(|o: Option<String>| match o.as_deref() {
        Some("") | None => Ok(None),
        Some(str) => str.parse::<Mime>().map(Some).map_err(|other| {
            serde::de::Error::invalid_value(
                Unexpected::Other(format!("{:?}", other).as_str()),
                &"valid mime type",
            )
        }),
    })
}

pub fn int_date_unix_timestamp_format<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let unix_timestamp = i64::deserialize(deserializer)?;
    Ok(Utc.timestamp(unix_timestamp, 0))
}

pub fn option_string_date_unix_timestamp_format<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::deserialize(deserializer).and_then(|o: Option<String>| match o.as_deref() {
        Some("0") | None => Ok(None),
        Some(str) => str
            .parse::<i64>()
            .map(|i| Some(Utc.timestamp(i, 0)))
            .map_err(serde::de::Error::custom),
    })
}

pub const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn option_string_date_format<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer)?.as_str() {
        "0000-00-00 00:00:00" => Ok(None),
        str => Utc
            .datetime_from_str(str, FORMAT)
            .map_err(serde::de::Error::custom)
            .map(Option::Some),
    }
}

// inspired by https://serde.rs/custom-date-format.html
pub mod string_date_unix_timestamp_format {
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
pub fn bool_to_int<S>(x: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let output = match x {
        true => "1",
        false => "0",
    };
    serializer.serialize_str(output)
}

pub fn borrow_url<S>(x: &Url, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(x.as_str())
}

pub fn true_to_unit_variant<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    if bool::deserialize(deserializer)? {
        Ok(())
    } else {
        Err(
            serde::de::Error::invalid_value(
                Unexpected::Bool(false),
                &r#"true"#
            )
        )
    }
}

pub fn false_to_unit_variant<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    if !bool::deserialize(deserializer)? {
        Ok(())
    } else {
        Err(
            serde::de::Error::invalid_value(
                Unexpected::Bool(false),
                &r#"false"#
            )
        )
    }
}