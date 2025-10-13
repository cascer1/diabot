#![allow(dead_code)]

use crate::util::deserializers::deserialize_glucose;
use serde::{Deserialize};
use crate::conversions::glucose::Glucose;
use crate::util::nightscout::types::TrendArrow;

#[derive(Deserialize, Debug)]
pub struct BgNowPlugin {
    #[serde(deserialize_with = "deserialize_glucose")]
    pub mean: Glucose,
    #[serde(deserialize_with = "deserialize_glucose")]
    pub last: Glucose,
    pub mills: i64,
    pub sgvs: Vec<Sgv>,
}

#[derive(Deserialize, Debug)]
pub struct Sgv {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_glucose")]
    pub mgdl: Glucose,
    #[serde(deserialize_with = "deserialize_glucose")]
    /// Scaled glucose value based on Nightscout unit setting
    pub scaled: Glucose,
    pub mills: i64,
    pub device: Option<String>,
    pub direction: TrendArrow,
    #[serde(rename = "type")]
    pub r#type: String,  // should always be "sgv"
}

#[derive(Deserialize, Debug)]
pub struct DeltaPlugin {
    #[serde(deserialize_with = "deserialize_glucose")]
    pub absolute: Glucose,
    #[serde(rename = "elapsedMins")]
    pub elapsed_mins: f64,
    pub interpolated: bool,
    #[serde(deserialize_with = "deserialize_glucose")]
    #[serde(rename = "mean5MinsAgo")]
    /// Mean glucose value for the last 5 minutes. Should be in mg/dL.
    pub mean_5m_ago: Glucose,
    pub times: DeltaTimes,
    #[serde(deserialize_with = "deserialize_glucose")]
    /// Glucose value in mg/dL
    pub mgdl: Glucose,
    #[serde(deserialize_with = "deserialize_glucose")]
    /// Scaled glucose value based on the Nightscout unit setting.
    pub scaled: Glucose,
    /// Display version of `scaled`. The unit is based on Nightscout settings.
    pub display: String,
    pub previous: DeltaPrevious,
}

#[derive(Deserialize, Debug)]
pub struct DeltaTimes {
    pub recent: i64,
    pub previous: i64,
}

#[derive(Deserialize, Debug)]
pub struct DeltaPrevious {
    #[serde(deserialize_with = "deserialize_glucose")]
    pub mean: Glucose,
    #[serde(deserialize_with = "deserialize_glucose")]
    pub last: Glucose,
    pub mills: i64,
    pub sgvs: Vec<Sgv>,
}

#[derive(Deserialize, Debug)]
pub struct DirectionPlugin {
    pub display: Option<String>,
    pub value: String,
    pub label: String,
    pub entity: String,
}

#[derive(Deserialize, Debug)]
pub struct IobPlugin {
    pub iob: f64,
    pub basaliob: f64,
    pub activity: f64,
    pub source: String,
    pub device: String,
    pub mills: i64,
    #[serde(rename = "treatmentIob")]
    pub treatment_iob: f64,
    pub display: String,
    #[serde(rename = "displayLine")]
    pub display_line: String,
}

#[derive(Deserialize, Debug)]
pub struct CobPlugin {
    pub cob: f64,
    pub source: String,
    pub device: String,
    pub mills: i64,
    pub display: f64,
    #[serde(rename = "displayLine")]
    pub display_line: String,
}

#[derive(Deserialize, Debug)]
pub struct NightscoutV2Properties {
    pub bgnow: BgNowPlugin,
    pub delta: DeltaPlugin,
    pub direction: DirectionPlugin,
    pub iob: Option<IobPlugin>,
    pub cob: Option<CobPlugin>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_nightscout_data() {
        let json_data = include_str!("properties_1.json");
        let parsed: NightscoutV2Properties = serde_json::from_str(json_data).expect("Failed to deserialize JSON");

        assert_eq!(parsed.bgnow.mean, Glucose::MgDl(252));
        assert_eq!(parsed.delta.absolute, Glucose::MgDl(-6));
        assert_eq!(parsed.direction.value, "Flat");
        assert!(parsed.iob.is_some());
        assert_eq!(parsed.iob.unwrap().iob, 4.198);
        assert!(parsed.cob.is_some());
        assert_eq!(parsed.cob.unwrap().cob, 0.0);

        assert_eq!(parsed.bgnow.sgvs.len(), 1);
        assert_eq!(parsed.bgnow.sgvs[0].mgdl, Glucose::MgDl(252));

        let json_data = include_str!("properties_2.json");
        let parsed: NightscoutV2Properties = serde_json::from_str(json_data).expect("Failed to deserialize JSON");
        assert_eq!(parsed.bgnow.sgvs.len(), 1);
        assert_eq!(parsed.bgnow.sgvs[0].mgdl, Glucose::MgDl(293));

        let json_data = include_str!("properties_3.json");
        let parsed: NightscoutV2Properties = serde_json::from_str(json_data).expect("Failed to deserialize JSON");
        assert_eq!(parsed.bgnow.sgvs.len(), 1);
        assert_eq!(parsed.bgnow.sgvs[0].mgdl, Glucose::MgDl(112));
    }

    #[test]
    fn test_bgnow_deserialization() {
        let json_data = r#"
        {
            "mean": 252,
            "last": 252,
            "mills": 1760297566932,
            "sgvs": [
                {
                    "_id": "68ec0262bbbb6deacc785a3d",
                    "mgdl": 252,
                    "mills": 1760297566932,
                    "device": "xDrip-DexcomG5",
                    "direction": "Flat",
                    "filtered": 0,
                    "unfiltered": 0,
                    "noise": 1,
                    "rssi": 100,
                    "type": "sgv",
                    "scaled": "14.0"
                }
            ]
        }
        "#;

        let parsed: BgNowPlugin = serde_json::from_str(json_data).expect("Failed to deserialize bgnow");

        assert_eq!(parsed.mean, Glucose::MgDl(252));
        assert_eq!(parsed.last, Glucose::MgDl(252));
        assert_eq!(parsed.mills, 1760297566932);
        assert_eq!(parsed.sgvs.len(), 1);

        let sgv = &parsed.sgvs[0];
        assert_eq!(sgv.mgdl, Glucose::MgDl(252));
        assert_eq!(sgv.device, Some("xDrip-DexcomG5".to_string()));
        assert_eq!(sgv.direction, TrendArrow::Flat);
    }

    #[test]
    fn test_delta_deserialization() {
        let json_data = r#"
        {
            "absolute": -6,
            "elapsedMins": 4.999616666666666,
            "interpolated": false,
            "mean5MinsAgo": 258,
            "times": {
                "recent": 1760297566932,
                "previous": 1760297266955
            },
            "mgdl": -6,
            "scaled": -0.3,
            "display": "-0.3",
            "previous": {
                "mean": 258,
                "last": 258,
                "mills": 1760297266955,
                "sgvs": [
                    {
                        "_id": "68ec0136bbbb6deacc785a3b",
                        "mgdl": 258,
                        "mills": 1760297266955,
                        "device": "xDrip-DexcomG5",
                        "direction": "Flat",
                        "filtered": 0,
                        "unfiltered": 0,
                        "noise": 1,
                        "rssi": 100,
                        "type": "sgv",
                        "scaled": "14.3"
                    }
                ]
            }
        }
        "#;

        let parsed: DeltaPlugin = serde_json::from_str(json_data).expect("Failed to deserialize delta");

        assert_eq!(parsed.absolute, Glucose::MgDl(-6));
        // unrounded floats my beloved
        assert_eq!(parsed.elapsed_mins, 4.999616666666666);
        assert_eq!(parsed.mgdl, Glucose::MgDl(-6));
        assert_eq!(parsed.scaled, Glucose::Mmol(-0.3));
        assert_eq!(parsed.display, "-0.3");
        assert_eq!(parsed.times.recent, 1760297566932);
        assert_eq!(parsed.times.previous, 1760297266955);

        let previous = &parsed.previous;
        assert_eq!(previous.mean, Glucose::MgDl(258));
        assert_eq!(previous.sgvs.len(), 1);

        let sgv = &previous.sgvs[0];
        assert_eq!(sgv.mgdl, Glucose::MgDl(258));
        assert_eq!(sgv.device, Some("xDrip-DexcomG5".to_string()));
    }


    #[test]
    fn test_direction_deserialization() {
        let json_data = r#"
        {
            "display": null,
            "value": "Flat",
            "label": "→",
            "entity": "&#8594;"
        }
        "#;

        let parsed: DirectionPlugin = serde_json::from_str(json_data).expect("Failed to deserialize direction");

        assert_eq!(parsed.value, "Flat");
        assert_eq!(parsed.label, "→");
        assert_eq!(parsed.entity, "&#8594;");
        assert!(parsed.display.is_none());
    }

    #[test]
    fn test_iob_deserialization() {
        let json_data = r#"
        {
            "iob": 4.198,
            "basaliob": 1.675,
            "activity": 0.0454,
            "source": "OpenAPS",
            "device": "openaps://Redacted",
            "mills": 1760297604111,
            "treatmentIob": 4.402,
            "display": "4.20",
            "displayLine": "IOB: 4.20U"
        }
        "#;

        let parsed: IobPlugin = serde_json::from_str(json_data).expect("Failed to deserialize iob");

        assert_eq!(parsed.iob, 4.198);
        assert_eq!(parsed.basaliob, 1.675);
        assert_eq!(parsed.activity, 0.0454);
        assert_eq!(parsed.source, "OpenAPS");
        assert_eq!(parsed.device, "openaps://Redacted");
        assert_eq!(parsed.display, "4.20");
        assert_eq!(parsed.display_line, "IOB: 4.20U");
    }


    #[test]
    fn test_cob_deserialization() {
        let json_data = r#"
        {
            "cob": 0,
            "source": "OpenAPS",
            "device": "openaps://Redacted",
            "mills": 1760297604111,
            "display": 0,
            "displayLine": "COB: 0g"
        }
        "#;

        let parsed: CobPlugin = serde_json::from_str(json_data).expect("Failed to deserialize cob");

        assert_eq!(parsed.cob, 0.0);
        assert_eq!(parsed.source, "OpenAPS");
        assert_eq!(parsed.device, "openaps://Redacted");
        assert_eq!(parsed.display, 0.0);
        assert_eq!(parsed.display_line, "COB: 0g");
    }

    #[test]
    fn test_invalid_json() {
        let invalid_json = r#"
        {
          "bgnow": {
            "mean": "invalid_data",
            "last": "invalid_data",
            "mills": 1760297566932,
            "sgvs": []
          },
          "delta": {}
        }
        "#;

        let result: Result<NightscoutV2Properties, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }
}
