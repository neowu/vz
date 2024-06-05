use std::fmt;

use serde::de;
use serde::Serialize;

use crate::util::exception::Exception;

pub fn from_json<'a, T>(json: &'a str) -> Result<T, Exception>
where
    T: de::Deserialize<'a>,
{
    serde_json::from_str(json).map_err(|err| Exception::unexpected_with_context(err, &format!("json={json}")))
}

pub fn to_json_pretty<T>(object: &T) -> Result<String, Exception>
where
    T: Serialize + fmt::Debug,
{
    serde_json::to_string_pretty(object).map_err(|err| Exception::unexpected_with_context(err, &format!("object={object:?}")))
}

pub fn to_json_value<T>(enum_value: &T) -> Result<String, Exception>
where
    T: Serialize + fmt::Debug,
{
    let value = serde_json::to_string(enum_value).map_err(|err| Exception::unexpected_with_context(err, &format!("enum={enum_value:?}")))?;
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
