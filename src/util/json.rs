use std::fmt;

use anyhow::Context;
use anyhow::Result;
use serde::de;
use serde::Serialize;

pub fn from_json<'a, T>(json: &'a str) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    serde_json::from_str(json).with_context(|| format!("json={json}"))
}

pub fn to_json_pretty<T>(object: &T) -> Result<String>
where
    T: Serialize + fmt::Debug,
{
    serde_json::to_string_pretty(object).with_context(|| format!("object={object:?}"))
}

pub fn to_json_value<T>(enum_value: &T) -> Result<String>
where
    T: Serialize + fmt::Debug,
{
    let value = serde_json::to_string(enum_value).with_context(|| format!("enum={enum_value:?}"))?;
    Ok(value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(|value| value.to_string())
        .unwrap_or(value))
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
        assert_eq!("macOS", json::to_json_value(&Os::MacOs).unwrap());
        assert_eq!("linux", json::to_json_value(&Os::Linux).unwrap());
    }
}
