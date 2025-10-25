use crate::util::deserializers::deserialize_glucose;
use crate::util::math::round_to;
use crate::util::nightscout::v1_models::GlucoseThreshold;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

const MGDL_PER_MMOL: f32 = 18.015588;
const MIN_BG_VALUE: f32 = -9999.0;
const MAX_BG_VALUE: f32 = 9999.0;

/// A glucose value and its unit of measurement.
#[derive(Debug, Clone, Copy)]
pub enum Glucose {
    MgDl(i32),
    Mmol(f32),
}

impl Glucose {
    /// Converts the value to mg/dL.
    /// If the value is already in mg/dL, it returns itself.
    pub fn to_mgdl(self) -> Glucose {
        match self {
            Glucose::MgDl(_) => self,
            Glucose::Mmol(val) => Glucose::MgDl((val * MGDL_PER_MMOL).round() as i32),
        }
    }

    /// Returns the glucose value in mg/dL, converting if needed.
    pub fn as_mgdl_value(&self) -> i32 {
        match self {
            Glucose::MgDl(val) => *val,
            Glucose::Mmol(val) => (*val * MGDL_PER_MMOL).round() as i32,
        }
    }

    /// Converts the value to mmol/L.
    /// If the value is already in mmol/L, it returns itself.
    pub fn to_mmol(self) -> Glucose {
        match self {
            Glucose::MgDl(val) => Glucose::Mmol(val as f32 / MGDL_PER_MMOL),
            Glucose::Mmol(_) => self,
        }
    }

    /// Returns the glucose value in mmol/L, converting if needed.
    pub fn as_mmol_value(&self) -> f32 {
        match self {
            Glucose::MgDl(val) => *val as f32 / MGDL_PER_MMOL,
            Glucose::Mmol(val) => *val,
        }
    }

    /// Convert this [Glucose] into the opposite unit.
    pub fn convert(&self) -> Glucose {
        match self {
            Glucose::MgDl(_) => self.to_mmol(),
            Glucose::Mmol(_) => self.to_mgdl(),
        }
    }

    /// Returns just the numeric part of the glucose value as a string.
    ///
    /// Useful for display purposes when the unit is implied or shown elsewhere.
    pub fn as_numeric_string(&self) -> String {
        match self {
            Glucose::MgDl(val) => format!("{}", val),
            Glucose::Mmol(val) => format!("{:.1}", val),
        }
    }

    /// Returns the glucose value as a signed delta string, prefixed with "+" if positive.
    ///
    /// Example: `+8`, `-0.4`
    pub fn as_delta_string(&self) -> String {
        let is_positive = self.as_mgdl_value() >= 0;
        format!(
            "{}{}",
            if is_positive { "+" } else { "" },
            self.as_numeric_string()
        )
    }

    /// Returns the glucose status based on the given threshold.
    pub fn status(&self, threshold: &GlucoseThreshold) -> GlucoseStatus {
        GlucoseStatus::new_status(self, threshold)
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

impl PartialEq for Glucose {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Compare values directly if both are in the same units
            (Glucose::MgDl(a), Glucose::MgDl(b)) => a == b,
            (Glucose::Mmol(a), Glucose::Mmol(b)) => round_to(*a, 1) == round_to(*b, 1),
            // If the units are different, convert them both to mg/dL and compare
            _ => self.as_mgdl_value() == other.as_mgdl_value(),
        }
    }
}

impl Eq for Glucose {}

impl Ord for Glucose {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            // Compare values directly if both are in the same units
            (Glucose::MgDl(a), Glucose::MgDl(b)) => a.cmp(b),
            (Glucose::Mmol(a), Glucose::Mmol(b)) => {
                let a_rounded = round_to(*a, 1);
                let b_rounded = round_to(*b, 1);
                a_rounded.partial_cmp(&b_rounded).unwrap_or(Ordering::Equal)
            }
            // If the units are different, convert them both to mg/dL and compare
            _ => self.as_mgdl_value().cmp(&other.as_mgdl_value()),
        }
    }
}

impl PartialOrd for Glucose {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'de> Deserialize<'de> for Glucose {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_glucose(deserializer)
    }
}

/// Represents whether a glucose value is within the configured BG thresholds/ranges.
#[derive(Debug, Clone, Copy)]
pub enum GlucoseStatus {
    /// Glucose is outside the low/high bounds.
    Urgent,

    /// Glucose is outside the target range but within the low/high bounds.
    Outside,

    /// Glucose is within the target range.
    Inside,
}

impl GlucoseStatus {
    /// Returns the glucose status based on the threshold ranges.
    ///
    /// * [`GlucoseStatus::Urgent`] if the glucose is below `bg_low` or above `bg_high`.
    /// * [`GlucoseStatus::Outside`] if the glucose is outside the target range but within high/low bounds.
    /// * [`GlucoseStatus::Inside`] if the glucose is within the target range.
    pub fn new_status(glucose: &Glucose, threshold: &GlucoseThreshold) -> GlucoseStatus {
        if *glucose >= threshold.bg_high || *glucose <= threshold.bg_low {
            GlucoseStatus::Urgent
        } else if *glucose >= threshold.bg_target_top || *glucose <= threshold.bg_target_bottom {
            GlucoseStatus::Outside
        } else {
            GlucoseStatus::Inside
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

    #[error("Number is out of range: {0} (between {min} and {max})", min = MIN_BG_VALUE, max = MAX_BG_VALUE)]
    OutOfRange(String),

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
        if !(MIN_BG_VALUE..=MAX_BG_VALUE).contains(&num) {
            return Err(ParseGlucoseError::OutOfRange(s.to_string()));
        }
        let num_int = num.round() as i32;

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
/// For validation, use [`ParsedGlucoseResult::parse`].
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
) -> Result<(f32, Option<String>), ParseGlucoseError> {
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
    let num: f32 = num_part
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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_known_parsed(input: &str, expected: Glucose) {
        let parsed = ParsedGlucoseResult::from_str(input).unwrap();
        match parsed {
            ParsedGlucoseResult::Known(bs) => {
                assert_eq!(bs, expected);
            }
            _ => panic!("Expected Known variant for input: {}", input),
        }
    }

    fn assert_ambiguous_parsed(
        input: &str,
        expected_original: &str,
        expected_mmol: Glucose,
        expected_mgdl: Glucose,
    ) {
        let parsed = ParsedGlucoseResult::from_str(input).unwrap();
        match parsed {
            ParsedGlucoseResult::Ambiguous {
                original,
                as_mmol,
                as_mgdl,
            } => {
                assert_eq!(original, expected_original);
                assert_eq!(as_mmol, expected_mmol);
                assert_eq!(as_mgdl, expected_mgdl);
            }
            _ => panic!("Expected Ambiguous variant for input: {}", input),
        }
    }

    mod conversions {
        use super::*;

        /// A helper function for comparing floats with a small tolerance.
        /// Direct comparison (`a == b`) with floating-point numbers can be unreliable
        /// due to precision issues.
        fn assert_approx_eq(a: f32, b: f32) {
            let epsilon = 1e-3;
            assert!(
                (a - b).abs() < epsilon,
                "Assertion failed: {} is not approximately equal to {}",
                a,
                b,
            );
        }

        #[test]
        fn test_mgdl_to_mmol() {
            let mgdl = Glucose::MgDl(100);
            let expected_mmol_val = 5.5507;

            if let Glucose::Mmol(val) = mgdl.to_mmol() {
                assert_approx_eq(val, expected_mmol_val);
            } else {
                panic!("Expected Glucose::Mmol");
            }
        }

        #[test]
        fn test_mmol_to_mgdl() {
            let mmol = Glucose::Mmol(5.5);
            let expected_mgdl_val = 99;

            assert_eq!(mmol.to_mgdl(), Glucose::MgDl(99));
            assert_eq!(expected_mgdl_val, 99);
        }

        #[test]
        fn test_rounding_mmol_to_mgdl() {
            // This value (100 / 18.015588) is ~5.5507, which should round up to 100 mg/dL
            let mmol = Glucose::Mmol(5.5507);
            assert_eq!(mmol.to_mgdl(), Glucose::MgDl(100));
        }

        #[test]
        fn test_idempotent_conversions() {
            // Calling a conversion on a value that is already in the target unit
            // should not change it.
            let mgdl = Glucose::MgDl(120);
            assert_eq!(mgdl.to_mgdl(), mgdl);

            let mmol = Glucose::Mmol(6.7);
            assert_eq!(mmol.to_mmol(), mmol);
        }

        #[test]
        fn test_general_convert_toggle() {
            let mgdl = Glucose::MgDl(150);
            let mmol = Glucose::Mmol(8.3);

            // Converting from mg/dL should yield mmol/L
            assert!(matches!(mgdl.convert(), Glucose::Mmol(_)));

            // Converting from mmol/L should yield mg/dL
            assert!(matches!(mmol.convert(), Glucose::MgDl(_)));
        }

        #[test]
        fn test_double_conversion_mgdl() {
            // Test if converting back and forth results in the original value.
            // Due to rounding, it should be very close but might not be exact.
            let original = Glucose::MgDl(125);
            let converted_back = original.convert().convert(); // MgDl -> Mmol -> MgDl

            assert_eq!(original, converted_back);
        }

        #[test]
        fn test_display_mgdl() {
            let glucose = Glucose::MgDl(120);
            assert_eq!(glucose.to_string(), "120 mg/dL");
        }

        #[test]
        fn test_display_mmol() {
            let glucose = Glucose::Mmol(6.4);
            assert_eq!(glucose.to_string(), "6.4 mmol/L");
        }

        #[test]
        fn test_display_mmol_rounding() {
            let glucose = Glucose::Mmol(5.67834);
            // Should round to 1 decimal place
            assert_eq!(glucose.to_string(), "5.7 mmol/L");
        }

        #[test]
        fn test_display_mmol_trailing_zero() {
            let glucose = Glucose::Mmol(7.0);
            // Should include one decimal place
            assert_eq!(glucose.to_string(), "7.0 mmol/L");
        }

        #[test]
        fn test_mgdl_equality() {
            let a = Glucose::MgDl(100);
            let b = Glucose::MgDl(100);
            let c = Glucose::MgDl(99);

            assert_eq!(a, b);
            assert_ne!(a, c);
            assert!(c < a);
        }

        #[test]
        fn test_mmol_equality_with_rounding() {
            let a = Glucose::Mmol(5.52); // rounds to 5.5
            let b = Glucose::Mmol(5.48); // rounds to 5.5
            let c = Glucose::Mmol(5.44); // rounds to 5.4

            assert_eq!(a, b);
            assert_ne!(a, c);
            assert!(c < a);
        }

        #[test]
        fn test_mgdl_vs_mmol_comparison() {
            let a = Glucose::MgDl(99); // 99 mg/dL
            let b = Glucose::Mmol(5.5); // 5.5 mmol/L ≈ 99.0 mg/dL
            let c = Glucose::Mmol(5.6); // ≈ 100.8 mg/dL
            let d = Glucose::MgDl(100); // 100 mg/dL

            assert_eq!(a, b);
            assert!(a < c);
            assert!(d < c); // 100 < 100.8
        }

        #[test]
        fn test_ord_trait_sorting() {
            let mut values = vec![
                Glucose::MgDl(90),
                Glucose::Mmol(5.0), // 90 mg/dL
                Glucose::Mmol(5.1), // 91.8 mg/dL
                Glucose::MgDl(100),
            ];

            values.sort();

            assert_eq!(values[0], Glucose::MgDl(90));
            assert_eq!(values[1], Glucose::Mmol(5.0));
            assert_eq!(values[2], Glucose::Mmol(5.1));
            assert_eq!(values[3], Glucose::MgDl(100));
        }

        #[test]
        fn test_floating_point_precision() {
            // Check values that are extremely close to rounding edge
            let a = Glucose::Mmol(5.149); // rounds to 5.1
            let b = Glucose::Mmol(5.151); // rounds to 5.2

            assert_ne!(a, b);
            assert!(a < b);
        }
    }

    mod parsing {
        use super::*;

        #[test]
        fn parse_known_mmol() {
            assert_known_parsed("5.2 mmol", Glucose::Mmol(5.2));
        }

        #[test]
        fn parse_known_mgdl() {
            assert_known_parsed("100 mg/dl", Glucose::MgDl(100));
        }

        #[test]
        fn parse_unambiguous_mmol_no_unit() {
            assert_known_parsed("4.8", Glucose::Mmol(4.8));
        }

        #[test]
        fn parse_unambiguous_mgdl_no_unit() {
            assert_known_parsed("180", Glucose::MgDl(180));
        }

        #[test]
        fn parse_ambiguous_no_unit() {
            assert_ambiguous_parsed("35", "35", Glucose::Mmol(35.0), Glucose::MgDl(35));
        }

        #[test]
        fn parse_unknown_unit() {
            let err = ParsedGlucoseResult::from_str("5.5 tests").unwrap_err();
            assert_eq!(err, ParseGlucoseError::UnknownUnit("tests".into()));
        }

        #[test]
        fn test_case_insensitive_and_alias_units() {
            let test_cases = [
                ("6.3 MMOL/L", Glucose::Mmol(6.3)),
                ("6.3 mmol", Glucose::Mmol(6.3)),
                ("6.3MMOL", Glucose::Mmol(6.3)),
                ("115 MG/dl", Glucose::MgDl(115)),
                ("115 mgdl", Glucose::MgDl(115)),
                ("115 mg", Glucose::MgDl(115)),
                ("115mgdl", Glucose::MgDl(115)),
            ];

            for (input, expected) in test_cases {
                let parsed = ParsedGlucoseResult::from_str(input).unwrap();
                match parsed {
                    ParsedGlucoseResult::Known(bs) => {
                        assert_eq!(bs, expected, "Failed on input: {}", input);
                    }
                    _ => panic!("Expected Known variant for input: {}", input),
                }
            }
        }

        #[test]
        fn parse_negative_and_zero_inputs() {
            assert_known_parsed("0 mmol", Glucose::Mmol(0.0));
            assert_known_parsed("-5 mg/dl", Glucose::MgDl(-5));
            assert_known_parsed("-5.5 mmol", Glucose::Mmol(-5.5));
        }

        #[test]
        fn parse_large_value_input() {
            assert_known_parsed("9999 mgdl", Glucose::MgDl(9999));
            assert_known_parsed("-9999 mmol", Glucose::Mmol(-9999.0));

            let err = ParsedGlucoseResult::from_str("10000 mgdl").unwrap_err();
            assert_eq!(err, ParseGlucoseError::OutOfRange("10000 mgdl".into()));

            let err = ParsedGlucoseResult::from_str("-10000 mmol").unwrap_err();
            assert_eq!(err, ParseGlucoseError::OutOfRange("-10000 mmol".into()));
        }

        #[test]
        fn parse_input_with_typos_or_spacing_errors() {
            let err = ParsedGlucoseResult::from_str("5.5 mmoll").unwrap_err();
            assert_eq!(err, ParseGlucoseError::UnknownUnit("mmoll".into()));

            let err = ParsedGlucoseResult::from_str("5.5 mmol / L ").unwrap_err();
            assert_eq!(err, ParseGlucoseError::UnknownUnit("mmol / l".into()));
        }
    }

    mod parse_glucose_str_input {
        use super::*;

        #[test]
        fn test_parse_glucose_input() {
            let cases = [
                ("5.5 mmol", (5.5, Some("mmol"))),
                ("5.5mmol/l", (5.5, Some("mmol/l"))),
                ("5.5mmol/L", (5.5, Some("mmol/l"))),
                ("5.5 mmol/L", (5.5, Some("mmol/l"))),
                ("180mg/dl", (180.0, Some("mg/dl"))),
                ("180 mg/dl", (180.0, Some("mg/dl"))),
                ("180mgdl", (180.0, Some("mgdl"))),
                ("180 mg", (180.0, Some("mg"))),
                ("180 MG/DL", (180.0, Some("mg/dl"))),
                ("180 randomunit", (180.0, Some("randomunit"))),
                ("180 Random Unit", (180.0, Some("random unit"))),
                ("5.5", (5.5, None)),
                ("180", (180.0, None)),
            ];

            for (input, expected) in cases {
                let parsed = parse_glucose_input(input, None).unwrap();
                assert_eq!(
                    parsed,
                    (expected.0, expected.1.map(|s| s.to_string())),
                    "Failed on input: {}",
                    input
                );
            }
        }

        #[test]
        fn test_parse_with_extra_spaces() {
            assert_eq!(
                parse_glucose_input("  7.1   mmol/L ", None).unwrap(),
                (7.1, Some("mmol/l".to_string()))
            );
        }

        #[test]
        fn test_parse_invalid_number() {
            assert_eq!(
                parse_glucose_input("abc mg/dl", None).unwrap_err(),
                ParseGlucoseError::InvalidNumber("abc mg/dl".into())
            );

            assert_eq!(
                parse_glucose_input("abc", None).unwrap_err(),
                ParseGlucoseError::InvalidNumber("abc".into())
            );
        }

        #[test]
        fn test_parse_empty_input() {
            assert_eq!(
                parse_glucose_input("", None).unwrap_err(),
                ParseGlucoseError::EmptyInput
            );
        }
    }
}

#[derive(Debug, Clone, poise::ChoiceParameter, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GlucoseUnit {
    #[name = "mg/dL"]
    #[serde(alias = "mg/dl")]
    Mgdl,
    #[name = "mmol/L"]
    #[serde(alias = "mmol/l")]
    Mmol,
}
