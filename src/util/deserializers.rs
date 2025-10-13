use crate::conversions::glucose::Glucose;
use serde::de::{Error, Visitor};
use serde::{de, Deserializer};
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
