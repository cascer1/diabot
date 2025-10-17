use crate::conversions::glucose::Glucose;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::fmt::Formatter;

pub fn deserialize_numstr<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    struct NumberStringVisitor;

    impl<'de> Visitor<'de> for NumberStringVisitor {
        type Value = f32;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            formatter.write_str("a number (i64, f64, or a string representing a float)")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(value as f32)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(value as f32)
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(value as f32)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            value
                .parse::<f32>()
                .map_err(|_| E::custom("invalid float value"))
        }
    }

    deserializer.deserialize_any(NumberStringVisitor)
}

pub fn deserialize_numstr_option<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    struct NumberStringOptionVisitor;

    impl<'de> Visitor<'de> for NumberStringOptionVisitor {
        type Value = Option<f32>;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            formatter.write_str("a number (i64, f64, or a string representing a float) or null")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Some(value as f32))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Some(value as f32))
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Some(value as f32))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            value
                .parse::<f32>()
                .map(Some)
                .map_err(|_| E::custom("invalid float value"))
        }
    }

    deserializer.deserialize_any(NumberStringOptionVisitor)
}

/// Custom deserializer for Glucose
///
/// Assumes floats are mmol/L and whole numbers are mg/dL
pub fn deserialize_glucose<'de, D>(deserializer: D) -> Result<Glucose, D::Error>
where
    D: Deserializer<'de>,
{
    struct GlucoseVisitor;

    impl<'de> Visitor<'de> for GlucoseVisitor {
        type Value = Glucose;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a glucose value in mg/dL (i32), mmol/L (f32), or either (string)")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match value.try_into() {
                Ok(val) => Ok(Glucose::MgDl(val)),
                Err(_) => Err(E::custom("value out of range")),
            }
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match value.try_into() {
                Ok(val) => Ok(Glucose::MgDl(val)),
                Err(_) => Err(E::custom("value out of range")),
            }
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            let val = value as f32;

            if val.is_finite() {
                Ok(Glucose::Mmol(val))
            } else {
                Err(E::custom("value out of range"))
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            let decimal = value.contains('.');
            value
                .parse::<f32>()
                .map(|v| {
                    if decimal {
                        Glucose::Mmol(v)
                    } else {
                        Glucose::MgDl(v.round() as i32)
                    }
                })
                .map_err(Error::custom)
        }
    }

    deserializer.deserialize_any(GlucoseVisitor)
}

/// Deserializes empty JSON objects (`{}`) as `None`.
///
/// This helper is useful when APIs return empty objects instead of `null` for optional fields.
/// If the input is an empty object, it returns `Ok(None)`. Otherwise, it attempts to deserialize
/// the value into `T` and wraps it in `Some(...)`.
///
/// # Example
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// struct MyStruct {
///     field: i32,
/// }
///
/// #[derive(Deserialize, Debug)]
/// struct Wrapper {
///     #[serde(deserialize_with = "empty_object_is_none")]
///     item: Option<MyStruct>,
/// }
///
/// let data = r#"{ "item": {} }"#;
/// let result: Wrapper = serde_json::from_str(data).unwrap();
/// assert!(result.item.is_none());
/// ```
pub fn empty_object_is_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let v = serde_json::Value::deserialize(deserializer)?;
    if v.is_object() && v.as_object().unwrap().is_empty() {
        Ok(None)
    } else {
        Ok(Some(T::deserialize(v).map_err(Error::custom)?))
    }
}
