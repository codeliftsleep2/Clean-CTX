//! Preserve duplicate-field evidence rather than silently choosing the last
//! value. Deserialization keeps serde_json's normal recursion and number limits.
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Number, Value};
use std::fmt;

pub(super) struct CheckedJson {
    pub value: Value,
    pub duplicate_fields: usize,
}

impl CheckedJson {
    fn scalar(value: Value) -> Self {
        Self {
            value,
            duplicate_fields: 0,
        }
    }
}

impl<'de> Deserialize<'de> for CheckedJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(JsonVisitor)
    }
}

struct JsonVisitor;

impl<'de> Visitor<'de> for JsonVisitor {
    type Value = CheckedJson;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E: Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(CheckedJson::scalar(Value::Bool(value)))
    }

    fn visit_i64<E: Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(CheckedJson::scalar(Value::Number(value.into())))
    }

    fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(CheckedJson::scalar(Value::Number(value.into())))
    }

    fn visit_f64<E: Error>(self, value: f64) -> Result<Self::Value, E> {
        Number::from_f64(value)
            .map(|number| CheckedJson::scalar(Value::Number(number)))
            .ok_or_else(|| E::custom("invalid JSON number"))
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
        self.visit_string(value.to_owned())
    }

    fn visit_string<E: Error>(self, value: String) -> Result<Self::Value, E> {
        Ok(CheckedJson::scalar(Value::String(value)))
    }

    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        Ok(CheckedJson::scalar(Value::Null))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        let mut duplicate_fields: usize = 0;
        while let Some(value) = access.next_element::<CheckedJson>()? {
            duplicate_fields = duplicate_fields.saturating_add(value.duplicate_fields);
            values.push(value.value);
        }
        Ok(CheckedJson {
            value: Value::Array(values),
            duplicate_fields,
        })
    }

    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let mut values = Map::new();
        let mut duplicate_fields: usize = 0;
        while let Some((name, value)) = access.next_entry::<String, CheckedJson>()? {
            duplicate_fields = duplicate_fields.saturating_add(value.duplicate_fields);
            if values.insert(name, value.value).is_some() {
                duplicate_fields = duplicate_fields.saturating_add(1);
            }
        }
        Ok(CheckedJson {
            value: Value::Object(values),
            duplicate_fields,
        })
    }
}
