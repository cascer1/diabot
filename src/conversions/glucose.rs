use std::fmt;
use std::str::FromStr;
use thiserror::Error;

pub const MGDL_PER_MMOL: f64 = 18.015588;

/// A glucose value and its unit of measurement.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Glucose {
    MgDl(i32),
    Mmol(f64),
}

impl Glucose {
    /// Returns the value converted to mg/dL.
    /// If the value is already in mg/dL, it returns a clone of itself.
    #[allow(dead_code)]
    pub fn as_mgdl(&self) -> Glucose {
        match self {
            Glucose::MgDl(_) => *self,
            Glucose::Mmol(val) => Glucose::MgDl((val * MGDL_PER_MMOL).round() as i32),
        }
    }

    /// Returns the value converted to mmol/L.
    /// If the value is already in mmol/L, it returns a clone of itself.
    #[allow(dead_code)]
    pub fn as_mmol(&self) -> Glucose {
        match self {
            Glucose::MgDl(val) => Glucose::Mmol(*val as f64 / MGDL_PER_MMOL),
            Glucose::Mmol(_) => *self,
        }
    }

    /// Convert this [Glucose] into the opposite unit.
    #[allow(dead_code)]
    pub fn convert(&self) -> Glucose {
        match self {
            Glucose::MgDl(_) => self.as_mmol(),
            Glucose::Mmol(_) => self.as_mgdl(),
        }
    }
}

impl fmt::Display for Glucose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Glucose::MgDl(val) => write!(f, "{} mg/dL", val),
            Glucose::Mmol(val) => write!(f, "{:.1} mmol/L", val),
        }
    }
}

/// Represents the result of parsing a string containing a glucose value, which may have ambiguous units.
#[derive(Debug, PartialEq)]
pub enum ParsedGlucoseResult {
    /// The value and unit were determined unambiguously.
    Known(Glucose),
    /// The value falls in a range where the unit is unclear.
    Ambiguous {
        original: String,
        as_mmol: Glucose,
        as_mgdl: Glucose,
    },
}

#[derive(Debug, PartialEq, Error)]
pub enum ParseGlucoseError {
    #[error("Missing or empty input.")]
    EmptyInput,

    #[error("Invalid number format: '{0}'")]
    InvalidNumber(String),

    #[error("Negative not allowed: '{0}'")]
    NegativeNumber(String),

    #[error("Unknown unit specified: '{0}'")]
    UnknownUnit(String),
}

/// Implements parsing from a string (`&str`) into a [ParsedGlucoseResult].
///
/// This handles:
/// 1. Parsing the number as a float.
/// 2. Parsing an optional, case-insensitive unit (e.g. "mg/dl", "mmol").
/// 3. Guessing the unit if it's missing (large values = mg/dl, small values = mmol).
impl FromStr for ParsedGlucoseResult {
    type Err = ParseGlucoseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (num, unit) = parse_glucose_input(s)?;
        let num_int = num.round() as i32;

        if num < 0.0 {
            return Err(ParseGlucoseError::NegativeNumber(s.to_string()));
        }

        if unit.is_empty() {
            if (25.0..=50.0).contains(&num) {
                Ok(ParsedGlucoseResult::Ambiguous {
                    original: s.to_string(),
                    as_mmol: Glucose::Mmol(num),
                    as_mgdl: Glucose::MgDl(num_int),
                })
            } else if num < 25.0 {
                Ok(ParsedGlucoseResult::Known(Glucose::Mmol(num)))
            } else {
                Ok(ParsedGlucoseResult::Known(Glucose::MgDl(num_int)))
            }
        } else {
            match unit.as_str() {
                "mmol" | "mmol/l" => Ok(ParsedGlucoseResult::Known(Glucose::Mmol(num))),
                "mg" | "mg/dl" | "mgdl" => Ok(ParsedGlucoseResult::Known(Glucose::MgDl(num_int))),
                _ => Err(ParseGlucoseError::UnknownUnit(unit)),
            }
        }
    }
}

/// Parses a blood glucose value from a string into a (number, unit) tuple.
/// Supports both "5.5 mmol" and "5.5mmol" styles.
pub fn parse_glucose_input(s: &str) -> Result<(f64, String), ParseGlucoseError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(ParseGlucoseError::EmptyInput);
    }

    // Splitting by whitespace
    let mut parts = s.split_whitespace();
    let first = parts.next().ok_or(ParseGlucoseError::EmptyInput)?;
    let second = parts.next();

    // Case 1: "5.5 mmol", space delimited number and unit
    if let Some(unit) = second {
        let num: f64 = first
            .parse()
            .map_err(|_| ParseGlucoseError::InvalidNumber(first.to_string()))?;
        return Ok((num, unit.trim().to_lowercase()));
    }

    // Case 2: "5.5mmol", we need to split the number and unit manually
    let split_pos = s
        .char_indices()
        .find(|(_, c)| !c.is_ascii_digit() && *c != '.')
        .map(|(i, _)| i)
        .unwrap_or(s.len());

    if split_pos == 0 {
        return Err(ParseGlucoseError::InvalidNumber(s.to_string()));
    }

    let (num_str, unit) = s.split_at(split_pos);
    let num: f64 = num_str
        .parse()
        .map_err(|_| ParseGlucoseError::InvalidNumber(num_str.to_string()))?;
    Ok((num, unit.trim().to_lowercase()))
}
