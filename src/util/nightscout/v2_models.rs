#![allow(dead_code)]

use crate::util::deserializers::empty_object_is_none;
use crate::conversions::glucose::Glucose;
use crate::util::nightscout::types::TrendArrow;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BgNowPlugin {
    pub mean: Glucose,
    pub last: Glucose,
    pub mills: i64,
    pub sgvs: Vec<Sgv>,
}

#[derive(Deserialize, Debug)]
pub struct Sgv {
    #[serde(rename = "_id")]
    pub id: String,
    pub mgdl: Glucose,
    /// Scaled glucose value based on Nightscout unit setting
    pub scaled: Glucose,
    pub mills: i64,
    pub device: Option<String>,
    pub direction: TrendArrow,
    #[serde(rename = "type")]
    pub r#type: String, // should always be "sgv"
}

#[derive(Deserialize, Debug)]
pub struct DeltaPlugin {
    pub absolute: Glucose,
    #[serde(rename = "elapsedMins")]
    pub elapsed_mins: f64,
    pub interpolated: bool,
    #[serde(rename = "mean5MinsAgo")]
    /// Mean glucose value for the last 5 minutes. Should be in mg/dL.
    pub mean_5m_ago: Glucose,
    pub times: DeltaTimes,
    /// Glucose value in mg/dL
    pub mgdl: Glucose,
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
    pub mean: Glucose,
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
    #[serde(default, deserialize_with = "empty_object_is_none")]
    pub bgnow: Option<BgNowPlugin>,

    #[serde(default, deserialize_with = "empty_object_is_none")]
    pub delta: Option<DeltaPlugin>,

    #[serde(default, deserialize_with = "empty_object_is_none")]
    pub direction: Option<DirectionPlugin>,

    #[serde(default, deserialize_with = "empty_object_is_none")]
    pub iob: Option<IobPlugin>,

    #[serde(default, deserialize_with = "empty_object_is_none")]
    pub cob: Option<CobPlugin>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_properties_1() {
        let json_data = include_str!("properties_1.json");
        let props: NightscoutV2Properties = serde_json::from_str(json_data).expect("Failed to deserialize JSON");

        let bgnow = props.bgnow.as_ref().expect("Expected bgnow to be Some");
        let delta = props.delta.as_ref().expect("Expected delta to be Some");
        let direction = props.direction.as_ref().expect("Expected direction to be Some");

        assert_eq!(bgnow.mean, Glucose::MgDl(252));
        assert_eq!(delta.mgdl, Glucose::MgDl(-6));
        assert_eq!(direction.value, "Flat");

        let iob = props.iob.as_ref().expect("Expected iob to be Some");
        assert_eq!(iob.iob, 4.198);

        let cob = props.cob.as_ref().expect("Expected cob to be Some");
        assert_eq!(cob.cob, 0.0);

        assert_eq!(bgnow.sgvs.len(), 1);
        assert_eq!(bgnow.sgvs[0].mgdl, Glucose::MgDl(252));
    }

    #[test]
    fn test_properties_2() {
        let json_data = include_str!("properties_2.json");
        let props: NightscoutV2Properties = serde_json::from_str(json_data).expect("Failed to deserialize JSON");

        let bgnow = props.bgnow.as_ref().expect("Expected bgnow to be Some");
        assert_eq!(bgnow.sgvs.len(), 1);
        assert_eq!(bgnow.sgvs[0].mgdl, Glucose::MgDl(293));
    }

    #[test]
    fn test_properties_3() {
        let json_data = include_str!("properties_3.json");
        let props: NightscoutV2Properties = serde_json::from_str(json_data).expect("Failed to deserialize JSON");

        let bgnow = props.bgnow.as_ref().expect("Expected bgnow to be Some");
        assert_eq!(bgnow.sgvs.len(), 1);
        assert_eq!(bgnow.sgvs[0].mgdl, Glucose::MgDl(112));
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

        let plugin: BgNowPlugin = serde_json::from_str(json_data).expect("Failed to deserialize bgnow");

        assert_eq!(plugin.mean, Glucose::MgDl(252));
        assert_eq!(plugin.last, Glucose::MgDl(252));
        assert_eq!(plugin.mills, 1760297566932);
        assert_eq!(plugin.sgvs.len(), 1);

        let sgv = &plugin.sgvs[0];
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

        let plugin: DeltaPlugin = serde_json::from_str(json_data).expect("Failed to deserialize delta");

        assert_eq!(plugin.absolute, Glucose::MgDl(-6));
        // unrounded floats my beloved
        assert_eq!(plugin.elapsed_mins, 4.999616666666666);
        assert_eq!(plugin.mgdl, Glucose::MgDl(-6));
        assert_eq!(plugin.scaled, Glucose::Mmol(-0.3));
        assert_eq!(plugin.display, "-0.3");
        assert_eq!(plugin.times.recent, 1760297566932);
        assert_eq!(plugin.times.previous, 1760297266955);

        let previous = &plugin.previous;
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

        let plugin: DirectionPlugin = serde_json::from_str(json_data).expect("Failed to deserialize direction");

        assert_eq!(plugin.value, "Flat");
        assert_eq!(plugin.label, "→");
        assert_eq!(plugin.entity, "&#8594;");
        assert!(plugin.display.is_none());
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

        let plugin: IobPlugin = serde_json::from_str(json_data).expect("Failed to deserialize iob");

        assert_eq!(plugin.iob, 4.198);
        assert_eq!(plugin.basaliob, 1.675);
        assert_eq!(plugin.activity, 0.0454);
        assert_eq!(plugin.source, "OpenAPS");
        assert_eq!(plugin.device, "openaps://Redacted");
        assert_eq!(plugin.display, "4.20");
        assert_eq!(plugin.display_line, "IOB: 4.20U");
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

        let plugin: CobPlugin = serde_json::from_str(json_data).expect("Failed to deserialize cob");

        assert_eq!(plugin.cob, 0.0);
        assert_eq!(plugin.source, "OpenAPS");
        assert_eq!(plugin.device, "openaps://Redacted");
        assert_eq!(plugin.display, 0.0);
        assert_eq!(plugin.display_line, "COB: 0g");
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
