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
    pub fn as_mgdl(&self) -> Glucose {
        match self {
            Glucose::MgDl(_) => *self,
            Glucose::Mmol(val) => Glucose::MgDl((val * MGDL_PER_MMOL).round() as i32),
        }
    }

    /// Returns the value converted to mmol/L.
    /// If the value is already in mmol/L, it returns a clone of itself.
    pub fn as_mmol(&self) -> Glucose {
        match self {
            Glucose::MgDl(val) => Glucose::Mmol(*val as f64 / MGDL_PER_MMOL),
            Glucose::Mmol(_) => *self,
        }
    }

    /// Convert this [Glucose] into the opposite unit.
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

impl ParsedGlucoseResult {
    /// Parses a glucose value with an optional unit.
    ///
    /// If both the string and the parameter specify a unit,
    /// the parameter takes precedence.
    pub fn parse(s: &str, unit: Option<&str>) -> Result<Self, ParseGlucoseError> {
        let (num, parsed_unit) = parse_glucose_input(s, unit)?;
        let num_int = num.round() as i32;

        if num < 0.0 {
            return Err(ParseGlucoseError::NegativeNumber(s.to_string()));
        }

        match parsed_unit.as_deref() {
            None | Some("") => {
                // Guess unit
                if (25.0..=50.0).contains(&num) {
                    Ok(Self::Ambiguous {
                        original: s.trim().to_string(),
                        as_mmol: Glucose::Mmol(num),
                        as_mgdl: Glucose::MgDl(num_int),
                    })
                } else if num < 25.0 {
                    Ok(Self::Known(Glucose::Mmol(num)))
                } else {
                    Ok(Self::Known(Glucose::MgDl(num_int)))
                }
            }

            Some(unit) => {
                // Unit provided
                match unit.to_lowercase().as_str() {
                    "mmol" | "mmol/l" => Ok(Self::Known(Glucose::Mmol(num))),
                    "mg" | "mg/dl" | "mgdl" => Ok(Self::Known(Glucose::MgDl(num_int))),
                    _ => Err(ParseGlucoseError::UnknownUnit(unit.to_string())),
                }
            }
        }
    }
}

impl FromStr for ParsedGlucoseResult {
    type Err = ParseGlucoseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s, None)
    }
}

/// Parses a blood glucose value and its unit from string input, returning a `(number, unit)` tuple.
///
/// The unit string in the result is always lowercased.
/// This function only extracts the unit; it does not verify that it's valid.
/// For validation, use [`ParsedGlucoseResult::parse_with_unit`].
///
/// If both the value string and the `unit` parameter specify a unit,
/// the `unit` parameter takes precedence.
///
/// The following input styles are supported:
/// - `"5.5 mmol"` - space-delimited
/// - `"5.5mmol"` - compact
/// - `"5,5 mmol"` - comma as decimal separator
/// - `("5.5", Some("mmol"))` - unit parameter
/// - `("5.5", None)` - no unit provided
pub fn parse_glucose_input(
    value: &str,
    unit: Option<&str>,
) -> Result<(f64, Option<String>), ParseGlucoseError> {
    // Normalize commas (`5,5` -> `5.5`)
    let value = value.trim().replace(',', ".");
    if value.is_empty() {
        return Err(ParseGlucoseError::EmptyInput);
    }

    // Try to find where the number ends and the unit (if any) begins
    let split_pos = value
        .char_indices()
        .find(|(_, c)| !c.is_ascii_digit() && *c != '.' && *c != '-')
        .map(|(i, _)| i)
        .unwrap_or(value.len());
    if split_pos == 0 {
        return Err(ParseGlucoseError::InvalidNumber(value));
    }
    let (num_part, unit_part) = value.split_at(split_pos);
    // Trim both parts
    let num_part = num_part.trim();
    let unit_part = unit_part.trim();

    // Parse number
    let num: f64 = num_part
        .parse()
        .map_err(|_| ParseGlucoseError::InvalidNumber(num_part.to_string()))?;

    // Determine unit
    let final_unit = match unit {
        Some(u) => Some(u.trim().to_lowercase()), // unit parameter
        None if !unit_part.is_empty() => Some(unit_part.to_lowercase()), // unit from value string
        _ => None,
    };

    Ok((num, final_unit))
}
