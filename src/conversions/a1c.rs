use crate::conversions::a1c::EstimationError::{
    IntermediateCalulationError, MissingBloodGlucoseValue,
};
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
    #[error("Missing blood glucose value to estimate from")]
    MissingBloodGlucoseValue,

    #[error("Unable to calculate intermediate value for {0}")]
    IntermediateCalulationError(String),
}

impl A1cEstimation {
    pub fn as_dcct(&mut self) -> Result<Self, EstimationError> {
        if self.dcct.is_some() {
            return Ok(*self);
        }

        if self.glucose.is_none() {
            return Err(MissingBloodGlucoseValue);
        }

        self.dcct = Some((self.glucose.unwrap().as_mgdl_value() as f32 + 46.7) / 28.7);

        Ok(*self)
    }

    pub fn as_dcct_value(&mut self) -> Result<f32, EstimationError> {
        if self.dcct.is_some() {
            return Ok(self.dcct.unwrap());
        }

        Ok(self.as_dcct()?.dcct.unwrap())
    }

    pub fn as_ifcc(&mut self) -> Result<Self, EstimationError> {
        if self.ifcc.is_some() {
            return Ok(*self);
        }

        if self.glucose.is_none() {
            return Err(MissingBloodGlucoseValue);
        }

        let temp_dcct = self.as_dcct()?.dcct;

        if temp_dcct.is_none() {
            return Err(IntermediateCalulationError(String::from("ifcc")));
        }

        self.ifcc = Some((temp_dcct.unwrap() - 2.15) * 10.929);

        Ok(*self)
    }

    pub fn as_ifcc_value(&mut self) -> Result<f32, EstimationError> {
        if self.ifcc.is_some() {
            return Ok(self.ifcc.unwrap());
        }

        Ok(self.as_ifcc()?.ifcc.unwrap())
    }

    // dcct = 0.017 * fructosamine + 1.61
    // fructosamine = (dcct - 1.61) * 58.82
    pub fn as_fructosamine(&mut self) -> Result<Self, EstimationError> {
        if self.fructosamine.is_some() {
            return Ok(*self);
        }

        if self.glucose.is_none() {
            return Err(MissingBloodGlucoseValue);
        }

        let temp_dcct = self.as_dcct()?.dcct;

        if temp_dcct.is_none() {
            return Err(IntermediateCalulationError(String::from("fructosamine")));
        }

        self.fructosamine = Some((temp_dcct.unwrap() - 1.61) * 58.82);

        Ok(*self)
    }

    pub fn as_fructosamine_value(&mut self) -> Result<f32, EstimationError> {
        if self.fructosamine.is_some() {
            return Ok(self.fructosamine.unwrap());
        }

        Ok(self.as_fructosamine()?.fructosamine.unwrap())
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
            MissingBloodGlucoseValue
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
            MissingBloodGlucoseValue
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
            MissingBloodGlucoseValue
        );
    }
}
