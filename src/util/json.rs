use std::fmt;

use serde::de;
use serde::Serialize;

pub fn from_json<'a, T>(json: &'a str) -> T
where
    T: de::Deserialize<'a>,
{
    serde_json::from_str(json).unwrap_or_else(|err| panic!("failed to deserialize, json={json}, err={err}"))
}

pub fn to_json_pretty<T>(object: &T) -> String
where
    T: Serialize + fmt::Debug,
{
    serde_json::to_string_pretty(object).unwrap_or_else(|err| panic!("failed to serialize, object-{object:?}, err={err}"))
}

pub fn to_json_value<T>(enum_value: &T) -> String
where
    T: Serialize + fmt::Debug,
{
    let value = serde_json::to_string(enum_value).unwrap_or_else(|err| panic!("failed to serialize, enum={enum_value:?}, err={err}"));
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(|value| value.to_string())
        .unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::json;

    #[derive(Serialize, Debug)]
    pub enum Os {
        #[serde(rename = "linux")]
        Linux,
        #[serde(rename = "macOS")]
        MacOs,
    }

    #[test]
    fn to_json_value() {
        assert_eq!("macOS", json::to_json_value(&Os::MacOs));
        assert_eq!("linux", json::to_json_value(&Os::Linux));
    }
}
