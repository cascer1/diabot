use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, FromRepr, VariantNames};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    FromRepr,
    EnumString,
    VariantNames,
    Display,
    Serialize,
    Deserialize,
)]
pub enum TrendArrow {
    #[serde(rename = "NONE")]
    #[strum(to_string = "")]
    None,

    #[strum(to_string = "↟")]
    DoubleUp,

    #[strum(to_string = "↑")]
    SingleUp,

    #[strum(to_string = "↗")]
    FortyFiveUp,

    #[strum(to_string = "→")]
    Flat,

    #[strum(to_string = "↘")]
    FortyFiveDown,

    #[strum(to_string = "↓")]
    SingleDown,

    #[strum(to_string = "↡")]
    DoubleDown,

    #[serde(rename = "NOT COMPUTABLE")]
    #[strum(to_string = "↮")]
    NotComputable,

    #[serde(rename = "RATE OUT OF RANGE")]
    #[strum(to_string = "↺")]
    RateOutOfRange,
}
