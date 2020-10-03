use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
    fmt,
};

use js_int::Int;
use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};
use serde_json::{to_string as to_json_string, Value as JsonValue};

use super::Error;

#[derive(Clone, Eq, PartialEq)]
pub enum CanonicalJsonValue {
    /// Represents a JSON null value.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use std::convert::TryInto;
    /// # use ruma_serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!(null).try_into().unwrap();
    /// ```
    Null,

    /// Represents a JSON boolean.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use std::convert::TryInto;
    /// # use ruma_serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!(true).try_into().unwrap();
    /// ```
    Bool(bool),

    /// Represents a JSON integer.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use std::convert::TryInto;
    /// # use ruma_serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!(12).try_into().unwrap();
    /// ```
    Integer(Int),

    /// Represents a JSON string.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use std::convert::TryInto;
    /// # use ruma_serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!("a string").try_into().unwrap();
    /// ```
    String(String),

    /// Represents a JSON array.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use std::convert::TryInto;
    /// # use ruma_serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!(["an", "array"]).try_into().unwrap();
    /// ```
    Array(Vec<CanonicalJsonValue>),

    /// Represents a JSON object.
    ///
    /// The map is backed by a BTreeMap to guarantee the sorting of keys.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use std::convert::TryInto;
    /// # use ruma_serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!({ "an": "object" }).try_into().unwrap();
    /// ```
    Object(BTreeMap<String, CanonicalJsonValue>),
}

impl fmt::Debug for CanonicalJsonValue {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Null => formatter.debug_tuple("Null").finish(),
            Self::Bool(v) => formatter.debug_tuple("Bool").field(&v).finish(),
            Self::Integer(ref v) => fmt::Debug::fmt(v, formatter),
            Self::String(ref v) => formatter.debug_tuple("String").field(v).finish(),
            Self::Array(ref v) => {
                formatter.write_str("Array(")?;
                fmt::Debug::fmt(v, formatter)?;
                formatter.write_str(")")
            }
            Self::Object(ref v) => {
                formatter.write_str("Object(")?;
                fmt::Debug::fmt(v, formatter)?;
                formatter.write_str(")")
            }
        }
    }
}

impl fmt::Display for CanonicalJsonValue {
    /// Display a JSON value as a string.
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let json = json!({ "city": "London", "street": "10 Downing Street" });
    ///
    /// // Canonical format:
    /// //
    /// // {"city":"London","street":"10 Downing Street"}
    /// let compact = format!("{}", json);
    /// assert_eq!(compact,
    ///     "{\"city\":\"London\",\"street\":\"10 Downing Street\"}");
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", to_json_string(&self).map_err(|_| fmt::Error)?)
    }
}

impl TryFrom<JsonValue> for CanonicalJsonValue {
    type Error = Error;

    fn try_from(json: JsonValue) -> Result<Self, Self::Error> {
        Ok(match json {
            JsonValue::Bool(b) => Self::Bool(b),
            JsonValue::Number(num) => Self::Integer(
                Int::try_from(num.as_i64().ok_or(Error::IntConvert)?)
                    .map_err(|_| Error::IntConvert)?,
            ),
            JsonValue::Array(vec) => {
                Self::Array(vec.into_iter().map(TryInto::try_into).collect::<Result<Vec<_>, _>>()?)
            }
            JsonValue::String(string) => Self::String(string),
            JsonValue::Object(obj) => Self::Object(
                obj.into_iter()
                    .map(|(k, v)| Ok((k, v.try_into()?)))
                    .collect::<Result<BTreeMap<_, _>, _>>()?,
            ),
            JsonValue::Null => Self::Null,
        })
    }
}

impl Serialize for CanonicalJsonValue {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(b) => serializer.serialize_bool(*b),
            Self::Integer(n) => n.serialize(serializer),
            Self::String(s) => serializer.serialize_str(s),
            Self::Array(v) => v.serialize(serializer),
            Self::Object(m) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for CanonicalJsonValue {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<CanonicalJsonValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = JsonValue::deserialize(deserializer)?;
        Ok(val.try_into().map_err(serde::de::Error::custom)?)
    }
}
