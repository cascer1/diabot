#![allow(dead_code)]

use crate::conversions::glucose::Glucose;
use crate::util::deserializers::empty_object_is_none;
use crate::util::nightscout::types::TrendArrow;
use serde::Deserialize;

/// Represents the most recent blood glucose reading(s) available,
/// obtained from the `/api/v2/properties/bgnow` endpoint.
///
/// # References
/// - [`bgnow.js`](https://github.com/nightscout/cgm-remote-monitor/blob/91cd601038a3bce00f92655cc6b1cc02fa3589d3/lib/plugins/bgnow.js)
#[derive(Deserialize, Debug)]
pub struct BgNowPlugin {
    /// The average (mean) glucose value from the recent SGVs in the bucket. Always in mg/dL.
    ///
    /// Calculated by summing all valid SGVs and dividing by the number of entries.
    pub mean: Glucose,

    /// The most recent glucose reading value. Always in mg/dL.
    ///
    /// This corresponds to the SGV with the latest timestamp (`mills`) in the dataset.
    pub last: Glucose,

    /// Timestamp of the most recent glucose reading, in milliseconds since epoch.
    pub mills: i64,

    /// List of raw SGV entries used to calculate [`BgNowPlugin::mean`] and [`BgNowPlugin::last`].
    ///
    /// Each SGV contains the original glucose value, timestamp, device source,
    /// and other metadata such as direction.
    pub sgvs: Vec<Sgv>,
}

/// Represents a single SGV (Sensor Glucose Value) reading.
///
/// A rough outline of the fields can be found in the Nightscout source code and in the Swagger API documentation for the `Entry` schema.
///
/// Unfortunately, some uploaders/sources exclude fields listed in the schema.
/// The fields in this struct are the ones that seem present on most SGV types.
///
/// # References
/// - [`Entry` schema in Swagger](https://github.com/nightscout/cgm-remote-monitor/blob/91cd601038a3bce00f92655cc6b1cc02fa3589d3/lib/server/swagger.json#L902-L941)
/// - [`dataloader.js`](https://github.com/nightscout/cgm-remote-monitor/blob/91cd601038a3bce00f92655cc6b1cc02fa3589d3/lib/data/dataloader.js#L221-L230)
#[derive(Deserialize, Debug)]
pub struct Sgv {
    /// MongoDB document ID for this reading.
    #[serde(rename = "_id")]
    pub id: String,

    /// The raw glucose value in mg/dL.
    pub mgdl: Glucose,

    /// Scaled glucose value based on the unit configured in Nightscout environment settings.
    ///
    /// If Nightscout is set to use mmol/L, this is the converted value from [`Sgv::mgdl`].
    /// Otherwise, it’s identical to [`Sgv::mgdl`].
    pub scaled: Glucose,

    /// Timestamp of this reading in milliseconds since epoch.
    pub mills: i64,

    /// Optional string identifying the device that collected this reading.
    pub device: Option<String>,

    /// The trend direction.
    pub direction: TrendArrow,

    /// The type of entry. This should always be `"sgv"`.
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Represents the calculated difference (delta) between two BG readings,
/// typically the most recent reading and the one preceding it.
///
/// **You'd usually want to use [`DeltaPlugin::mgdl`], [`DeltaPlugin::scaled`], or [`DeltaPlugin::display`] for displaying the delta value.**
///
/// This data is obtained from the `/api/v2/properties/delta` endpoint.
///
/// Although technically part of the bgnow plugin, this data is exposed as a separate set of properties in the API.
///
/// # References
/// - [`calcDelta`](https://github.com/nightscout/cgm-remote-monitor/blob/91cd601038a3bce00f92655cc6b1cc02fa3589d3/lib/plugins/bgnow.js#L147-L185)
#[derive(Deserialize, Debug)]
pub struct DeltaPlugin {
    /// The change in glucose between the current reading and reading 5 minutes ago. Always in mg/dL.
    ///
    /// May be interpolated if the time gap is large, see [`DeltaPlugin::absolute`] if you need the non-interpolated delta between the two readings.
    ///
    /// Computed as `recent.mean - mean_5m_ago`.
    pub mgdl: Glucose,

    /// Delta value converted (scaled) using the measurement unit set in the Nightscout environment settings.
    ///
    /// Calculated the same as [`DeltaPlugin::mgdl`], but applies a conversion to mmol/L if Nightscout is using mmol/L
    pub scaled: Glucose,

    /// Display version of [`DeltaPlugin::scaled`]. Includes a sign prefix ('+' or '-').
    ///
    /// The unit is based on Nightscout settings.
    pub display: String,

    /// The absolute change in glucose between the recent and previous readings. Always in mg/dL.
    ///
    /// Note that this represents the *absolute* delta, not the interpolated value.
    /// Nightscout (and other apps) usually display the interpolated delta, not the absolute delta.
    /// This is mainly for internal use and is seldom used for display.
    ///
    /// **If you want the delta for display purposes, see [`DeltaPlugin::mgdl`], [`DeltaPlugin::scaled`], or [`DeltaPlugin::display`].**
    ///
    /// This is calculated as: `recent.mean - previous.mean`.
    pub absolute: Glucose,

    /// Estimated glucose value from 5 minutes ago. Always in mg/dL.
    ///
    /// If [`DeltaPlugin::interpolated`] is `true`, it's calculated as: `recent.mean - delta.absolute / delta.elapsedMins * 5`.
    ///
    /// Otherwise, it's calculated as: `recent.mean - delta.absolute`.
    #[serde(rename = "mean5MinsAgo")]
    pub mean_5m_ago: Glucose,

    /// Timestamp information in milliseconds since epoch.
    pub times: DeltaTimes,

    /// The number of minutes elapsed between the recent and previous data points.
    ///
    /// This is calculated as: `(recent.mills - previous.mills) / 60000`.
    #[serde(rename = "elapsedMins")]
    pub elapsed_mins: f64,

    /// Whether the delta had to be interpolated due to a large time gap.
    ///
    /// This is calculated with: `elapsed_mins > 9`.
    pub interpolated: bool,

    /// Copy of the previous glucose data.
    pub previous: BgNowPlugin,
}

/// Timestamps (in milliseconds since epoch) for the two glucose readings used in [`DeltaPlugin`].
///
/// References:
/// - [`calcDelta`](https://github.com/nightscout/cgm-remote-monitor/blob/91cd601038a3bce00f92655cc6b1cc02fa3589d3/lib/plugins/bgnow.js#L147-L185)
#[derive(Deserialize, Debug)]
pub struct DeltaTimes {
    /// Timestamp of the most recent glucose reading.
    pub recent: i64,

    /// Timestamp of the previous glucose reading.
    pub previous: i64,
}

/// Represents the calculated glucose trend direction and its associated display symbols,
/// obtained from the `/api/v2/properties/direction` endpoint.
///
/// # References
/// - [`direction.js`](https://github.com/nightscout/cgm-remote-monitor/blob/91cd601038a3bce00f92655cc6b1cc02fa3589d3/lib/plugins/direction.js)
#[derive(Deserialize, Debug)]
pub struct DirectionPlugin {
    /// Trend direction from the SGV (e.g. `"Flat"`, `"DoubleUp"`, `"FortyFiveDown"`), deserialized into a [`TrendArrow`] enum.
    ///
    /// This should be the same as [`Sgv::direction`].
    pub value: TrendArrow,

    /// Unicode character representing the trend direction (e.g. `'→'`, `'⇈'`, `'↘'`).
    pub label: String,

    /// HTML-encoded entity version of the [`DirectionPlugin::label`] field (e.g. `'&#8592;'` for `'→'`).
    pub entity: String,
}

/// Represents calculated insulin-on-board (IOB) values derived from treatments and devicestatus.
///
/// This data is obtained from the `/api/v2/properties/iob` endpoint, which merges IOB contributions
/// from devicestatus (Loop, OpenAPS, etc.) and treatments.
///
/// Depending on the source, some fields may be excluded.
///
/// # References
/// - [`iob.js`](https://github.com/nightscout/cgm-remote-monitor/blob/91cd601038a3bce00f92655cc6b1cc02fa3589d3/lib/plugins/iob.js)
#[derive(Deserialize, Debug)]
pub struct IobPlugin {
    /// Total insulin on board (basal and bolus) at the current time, in units (U).
    pub iob: f64,

    /// Portion of [`IobPlugin::iob`] attributed to basal insulin, in units.
    ///
    /// Only present if the source is OpenAPS.
    #[serde(rename = "basaliob")]
    pub basal_iob: Option<f64>,

    /// Insulin activity level.
    ///
    /// Only present if the source is OpenAPS or Care Portal.
    pub activity: Option<f64>,

    /// IOB value calculated solely from treatment records (Care Portal), in units.
    ///
    /// Only present if the source is **not** Care Portal.
    #[serde(rename = "treatmentIob")]
    pub treatment_iob: Option<f64>,

    /// Timestamp of when the IOB data was calculated, in milliseconds since epoch.
    ///
    /// Only present if the source is **not** Care Portal.
    pub mills: Option<i64>,

    /// Source of the IOB data.
    ///
    /// Only present if the source is `"Care Portal"`, `"OpenAPS"`, `"Loop"`, or `"MM Connect"`.
    ///
    /// IOB data can still be provided by devicestatus entries with a generic `pump` field, but this would be `None`.
    pub source: Option<String>,

    /// Device identifier which uploaded the IOB data.
    ///
    /// Should match the `device` field in devicestatus.
    ///
    /// Only present if the source is **not** Care Portal
    pub device: Option<String>,

    /// IOB value formatted to 2 decimal places for display (`"0.80"`).
    pub display: Option<String>,

    /// Display line for the UI, including label and unit (`"IOB: 0.80U"`).
    #[serde(rename = "displayLine")]
    pub display_line: Option<String>,
}

/// Represents calculated carbohydrates-on-board (COB) values derived from treatments and devicestatus.
///
/// This data is obtained from the `/api/v2/properties/cob` endpoint, which retrieves COB data
/// from either devicestatus (Loop, OpenAPS, etc.) or treatments.
///
/// Depending on the source, some fields may be excluded.
///
/// # References
/// - [`cob.js`](https://github.com/nightscout/cgm-remote-monitor/blob/91cd601038a3bce00f92655cc6b1cc02fa3589d3/lib/plugins/cob.js)
#[derive(Deserialize, Debug)]
pub struct CobPlugin {
    /// Carbs-on-board in grams (g).
    pub cob: f64,

    /// Timestamp of when the COB calculation was done, in milliseconds since epoch.
    ///
    /// Missing when source is "Care Portal".
    pub mills: Option<i64>,

    /// Source of the COB data (e.g. `"OpenAPS"`, `"Loop"`, `"Care Portal"`).
    pub source: Option<String>,

    /// Device identifier which uploaded the COB data.
    pub device: Option<String>,

    /// COB value rounded to 1 decimal place for display (`15.2`).
    pub display: Option<f64>,

    /// Display line for the UI, including label and unit (`"COB: 15.2g"`).
    #[serde(rename = "displayLine")]
    pub display_line: Option<String>,
}

/// Represents the response payload from the Nightscout `/api/v2/properties` endpoint.
///
/// Each field is optional depending on available data.
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
        let props: NightscoutV2Properties =
            serde_json::from_str(json_data).expect("Failed to deserialize JSON");

        let bgnow = props.bgnow.as_ref().expect("Expected bgnow to be Some");
        let delta = props.delta.as_ref().expect("Expected delta to be Some");
        let direction = props
            .direction
            .as_ref()
            .expect("Expected direction to be Some");

        assert_eq!(bgnow.mean, Glucose::MgDl(252));
        assert_eq!(delta.mgdl, Glucose::MgDl(-6));
        assert_eq!(direction.value, TrendArrow::Flat);

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
        let props: NightscoutV2Properties =
            serde_json::from_str(json_data).expect("Failed to deserialize JSON");

        let bgnow = props.bgnow.as_ref().expect("Expected bgnow to be Some");
        assert_eq!(bgnow.sgvs.len(), 1);
        assert_eq!(bgnow.sgvs[0].mgdl, Glucose::MgDl(293));
    }

    #[test]
    fn test_properties_3() {
        let json_data = include_str!("properties_3.json");
        let props: NightscoutV2Properties =
            serde_json::from_str(json_data).expect("Failed to deserialize JSON");

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

        let plugin: BgNowPlugin =
            serde_json::from_str(json_data).expect("Failed to deserialize bgnow");

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

        let plugin: DeltaPlugin =
            serde_json::from_str(json_data).expect("Failed to deserialize delta");

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

        let plugin: DirectionPlugin =
            serde_json::from_str(json_data).expect("Failed to deserialize direction");

        assert_eq!(plugin.value, TrendArrow::Flat);
        assert_eq!(plugin.label, "→");
        assert_eq!(plugin.entity, "&#8594;");
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
        assert_eq!(plugin.basal_iob, Some(1.675));
        assert_eq!(plugin.activity, Some(0.0454));
        assert_eq!(plugin.source, Some("OpenAPS".into()));
        assert_eq!(plugin.device, Some("openaps://Redacted".into()));
        assert_eq!(plugin.display, Some("4.20".into()));
        assert_eq!(plugin.display_line, Some("IOB: 4.20U".into()));
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
        assert_eq!(plugin.source, Some("OpenAPS".into()));
        assert_eq!(plugin.device, Some("openaps://Redacted".into()));
        assert_eq!(plugin.display, Some(0.0));
        assert_eq!(plugin.display_line, Some("COB: 0g".into()));
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
