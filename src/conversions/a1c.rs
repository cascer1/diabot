use crate::conversions::a1c::EstimationError::{IntermediateCalulationError, MissingInputValue};
use crate::conversions::glucose::Glucose;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
struct A1cEstimation {
    glucose: Option<Glucose>,
    ifcc: Option<f32>,
    dcct: Option<f32>,
    fructosamine: Option<f32>,
}

#[derive(Debug, PartialEq, Error)]
pub enum EstimationError {
    #[error("Unable to calculate {0}, expected input value(s): {1}")]
    MissingInputValue(String, String),
}

impl A1cEstimation {
    fn calculate_dcct(&mut self) -> Result<Self, EstimationError> {
        if self.dcct.is_some() {
            return Ok(*self);
        }

        if self.glucose.is_some() {
            self.dcct = Some((self.glucose.unwrap().as_mgdl_value() as f32 + 46.7) / 28.7);
        } else if self.ifcc.is_some() {
            // dcct = (ifcc/10.929)+2.15
            self.dcct = Some((self.ifcc.unwrap() / 10.929) + 2.15)
        } else {
            return Err(MissingInputValue(
                String::from("dcct"),
                String::from("glucose, ifcc"),
            ));
        }

        Ok(*self)
    }

    pub fn as_dcct_value(&mut self) -> Result<f32, EstimationError> {
        if self.dcct.is_some() {
            return Ok(self.dcct.unwrap());
        }

        Ok(self.calculate_dcct()?.dcct.unwrap())
    }

    fn calculate_ifcc(&mut self) -> Result<Self, EstimationError> {
        if self.ifcc.is_some() {
            return Ok(*self);
        }

        if self.dcct.is_some() {
            self.ifcc = Some((self.dcct.unwrap() - 2.15) * 10.929)
        } else if self.glucose.is_some() {
            self.ifcc = Some((self.calculate_dcct().unwrap().dcct.unwrap() - 2.15) * 10.929)
        } else {
            return Err(MissingInputValue(
                "ifcc".to_string(),
                "glucose, dcct".to_string(),
            ));
        }

        Ok(*self)
    }

    pub fn as_ifcc_value(&mut self) -> Result<f32, EstimationError> {
        if self.ifcc.is_some() {
            return Ok(self.ifcc.unwrap());
        }

        Ok(self.calculate_ifcc()?.ifcc.unwrap())
    }

    // dcct = 0.017 * fructosamine + 1.61
    // fructosamine = (dcct - 1.61) * 58.82
    fn calculate_fructosamine(&mut self) -> Result<Self, EstimationError> {
        if self.fructosamine.is_some() {
            return Ok(*self);
        }

        if self.dcct.is_some() {
            self.fructosamine = Some((self.dcct.unwrap() - 1.61) * 58.82)
        } else if self.glucose.is_some() {
            self.fructosamine = Some((self.calculate_dcct()?.dcct.unwrap() - 1.61) * 58.82)
        } else {
            return Err(MissingInputValue("fructosamine".to_string(), "dcct, glucose".to_string()))
        }

        Ok(*self)
    }

    pub fn as_fructosamine_value(&mut self) -> Result<f32, EstimationError> {
        if self.fructosamine.is_some() {
            return Ok(self.fructosamine.unwrap());
        }

        Ok(self.calculate_fructosamine()?.fructosamine.unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_approx_eq(a: f32, b: f32) {
        assert_approx_eq_with_epsilon(a, b, 1e-3);
    }

    fn assert_approx_eq_with_epsilon(a: f32, b: f32, epsilon: f32) {
        assert!(
            (a - b).abs() < epsilon,
            "Assertion failed: {} is not approximately equal to {}",
            a,
            b
        )
    }

    #[test]
    fn test_glucose_mgdl_to_dcct() {
        let glucose = Glucose::MgDl(100);
        let expected_dcct = 5.111;

        let actual_dcct = A1cEstimation {
            glucose: Some(glucose),
            ifcc: None,
            dcct: None,
            fructosamine: None,
        }
        .as_dcct_value()
        .unwrap();

        assert_approx_eq(expected_dcct, actual_dcct);
    }

    #[test]
    fn test_glucose_mmol_to_dcct() {
        let glucose = Glucose::Mmol(5.6);
        // without intermediate rounding this would be 5.142
        let expected_dcct = 5.146;

        let actual_dcct = A1cEstimation {
            glucose: Some(glucose),
            ifcc: None,
            dcct: None,
            fructosamine: None,
        }
        .as_dcct_value()
        .unwrap();

        assert_approx_eq(expected_dcct, actual_dcct);
    }

    #[test]
    fn test_glucose_mgdl_to_ifcc() {
        let glucose = Glucose::MgDl(100);
        let expected = 32.366;

        let actual = A1cEstimation {
            glucose: Some(glucose),
            ifcc: None,
            dcct: None,
            fructosamine: None,
        }
        .as_ifcc_value()
        .unwrap();

        assert_approx_eq(expected, actual);
    }

    #[test]
    fn test_glucose_mmol_to_ifcc() {
        let glucose = Glucose::Mmol(5.6);
        let expected = 32.747;

        let actual = A1cEstimation {
            glucose: Some(glucose),
            ifcc: None,
            dcct: None,
            fructosamine: None,
        }
        .as_ifcc_value()
        .unwrap();

        assert_approx_eq(expected, actual);
    }

    #[test]
    fn test_dcct_to_ifcc() {
        let dcct = 6.7;
        let expected = 49.727;

        let actual = A1cEstimation {
            glucose: None,
            ifcc: None,
            dcct: Some(dcct),
            fructosamine: None,
        }
        .as_ifcc_value()
        .unwrap();

        assert_approx_eq(expected, actual);
    }

    #[test]
    fn test_glucose_mgdl_to_fructosamine() {
        let glucose = Glucose::MgDl(100);
        let expected = 205.9586;

        let actual = A1cEstimation {
            glucose: Some(glucose),
            ifcc: None,
            dcct: None,
            fructosamine: None,
        }
        .as_fructosamine_value()
        .unwrap();

        assert_approx_eq(expected, actual);
    }

    #[test]
    fn test_glucose_mmol_to_fructosamine() {
        let glucose = Glucose::Mmol(5.6);
        let expected = 208.008;

        let actual = A1cEstimation {
            glucose: Some(glucose),
            ifcc: None,
            dcct: None,
            fructosamine: None,
        }
        .as_fructosamine_value()
        .unwrap();

        assert_approx_eq(expected, actual);
    }

    #[test]
    fn test_calculate_dcct_without_input() {
        assert_eq!(
            A1cEstimation {
                glucose: None,
                ifcc: None,
                dcct: None,
                fructosamine: None,
            }
            .as_dcct_value()
            .unwrap_err(),
            MissingInputValue("dcct".to_string(), "glucose, ifcc".to_string())
        );
    }

    #[test]
    fn test_calculate_ifcc_without_input() {
        assert_eq!(
            A1cEstimation {
                glucose: None,
                ifcc: None,
                dcct: None,
                fructosamine: None,
            }
            .as_ifcc_value()
            .unwrap_err(),
            MissingInputValue("ifcc".to_string(), "glucose, dcct".to_string())
        );
    }

    #[test]
    fn test_calculate_fructosamine_without_input() {
        assert_eq!(
            A1cEstimation {
                glucose: None,
                ifcc: None,
                dcct: None,
                fructosamine: None,
            }
            .as_fructosamine_value()
            .unwrap_err(),
            MissingInputValue("fructosamine".to_string(), "dcct, glucose".to_string())
        );
    }
}
