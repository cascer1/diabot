#![allow(dead_code)]

use crate::util::deserializers::deserialize_numstr;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::serde_as;
use crate::conversions::glucose::GlucoseUnit;
use crate::util::nightscout::types::TrendArrow;
use crate::util::nightscout::v2_models::NightscoutV2Properties;

/// Combined result struct containing everything fetched.
#[derive(Debug, Deserialize)]
pub struct CombinedNightscout {
    // Optional entries (most recent first if requested with count)
    // pub entries: Option<Vec<Entry>>,

    /// /status response
    pub status: Status,

    pub properties: NightscoutV2Properties,

    // Optional /pebble response
    // pub pebble: Option<Pebble>,
}

/// A struct representing a SGV object from /api/v1/entries.json with common Nightscout fields and `extra`
/// for any unknown fields. `date` is epoch milliseconds.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entry {
    #[serde(default)]
    pub sgv: Option<i64>,

    #[serde(default)]
    pub date: i64,

    #[serde(default)]
    #[serde(rename = "dateString")]
    pub date_string: Option<String>,

    #[serde(default)]
    pub direction: Option<String>,

    /// Raw body structure for unexpected/additional fields
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, serde_query::Deserialize, Clone)]
pub struct Status {
    #[query(".status")]
    pub status: String,

    #[query(".settings.customTitle")]
    pub custom_title: String,

    #[query(".settings.units")]
    pub units: GlucoseUnit,

    // these should be in mg/dL
    #[query(".settings.thresholds.bgHigh")]
    pub bg_high: i32,

    #[query(".settings.thresholds.bgTargetTop")]
    pub bg_target_top: i32,

    #[query(".settings.thresholds.bgTargetBottom")]
    pub bg_target_bottom: i32,

    #[query(".settings.thresholds.bgLow")]
    pub bg_low: i32,
}

/// Pebble endpoint fields (cob, iob, bgdelta etc.)
#[derive(Debug, Serialize, Deserialize)]
pub struct Pebble {
    pub bgs: Vec<PebbleBgEntry>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PebbleBgEntry {
    #[serde(deserialize_with = "deserialize_numstr")]
    pub sgv: f32,

    pub trend: i32,
    pub direction: TrendArrow,
    pub datetime: i64,

    #[serde(deserialize_with = "deserialize_numstr")]
    pub bgdelta: f32,

    #[serde(deserialize_with = "deserialize_numstr")]
    pub battery: f32,

    #[serde(deserialize_with = "deserialize_numstr")]
    pub iob: f32,

    #[serde(deserialize_with = "deserialize_numstr")]
    pub bwp: f32,

    pub bwpo: f32,
    pub cob: i32,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pebble_deserialization() {
        let mgdl_json = r#"
{
  "status": [
    {
      "now": 1760221707039
    }
  ],
  "bgs": [
    {
      "sgv": "274",
      "trend": 3,
      "direction": "FortyFiveUp",
      "datetime": 1760221556907,
      "bgdelta": 7,
      "battery": "100",
      "iob": "2.62",
      "bwp": "-1.33",
      "bwpo": -18,
      "cob": 0
    }
  ],
  "cals": []
}
        "#;
        let expected_mgdl = PebbleBgEntry {
            sgv: 274.0,
            trend: 3,
            direction: TrendArrow::FortyFiveUp,
            datetime: 1760221556907,
            bgdelta: 7.0,
            battery: 100.0,
            iob: 2.62,
            bwp: -1.33,
            bwpo: -18.0,
            cob: 0,
        };

        let pebble: Pebble = serde_json::from_str(mgdl_json).unwrap();
        assert_eq!(pebble.bgs.len(), 1);
        assert_eq!(pebble.bgs[0], expected_mgdl);

        let mmol_json = r#"
{
  "status": [
    {
      "now": 1760221320322
    }
  ],
  "bgs": [
    {
      "sgv": "12.0",
      "trend": 5,
      "direction": "FortyFiveDown",
      "datetime": 1760221063233,
      "bgdelta": "-1.3",
      "battery": "28",
      "iob": "3.21",
      "bwp": "-0.21",
      "bwpo": 5.6,
      "cob": 0
    }
  ],
  "cals": []
}
        "#;
        let expected_mmol = PebbleBgEntry {
            sgv: 12.0,
            trend: 5,
            direction: TrendArrow::FortyFiveDown,
            datetime: 1760221063233,
            bgdelta: -1.3,
            battery: 28.0,
            iob: 3.21,
            bwp: -0.21,
            bwpo: 5.6,
            cob: 0,
        };
        let pebble: Pebble = serde_json::from_str(mmol_json).unwrap();
        assert_eq!(pebble.bgs.len(), 1);
        assert_eq!(pebble.bgs[0], expected_mmol);
    }

    #[test]
    fn test_status_deserialization() {
        let json = r#"
        {
  "status" : "ok",
  "name" : "nightscout",
  "version" : "15.0.2",
  "serverTime" : "2025-10-13T04:18:20.423Z",
  "serverTimeEpoch" : 1760329100423,
  "apiEnabled" : true,
  "careportalEnabled" : true,
  "boluscalcEnabled" : false,
  "settings" : {
    "units" : "mg/dl",
    "timeFormat" : 24,
    "dayStart" : 7,
    "dayEnd" : 21,
    "nightMode" : false,
    "editMode" : true,
    "showRawbg" : "never",
    "customTitle" : "Nightscout",
    "theme" : "colorblindfriendly",
    "alarmUrgentHigh" : false,
    "alarmUrgentHighMins" : [ 30, 60, 90, 120 ],
    "alarmHigh" : false,
    "alarmHighMins" : [ 30, 60, 90, 120 ],
    "alarmLow" : false,
    "alarmLowMins" : [ 15, 30, 45, 60 ],
    "alarmUrgentLow" : false,
    "alarmUrgentLowMins" : [ 15, 30, 45 ],
    "alarmUrgentMins" : [ 30, 60, 90, 120 ],
    "alarmWarnMins" : [ 30, 60, 90, 120 ],
    "alarmTimeagoWarn" : false,
    "alarmTimeagoWarnMins" : "15",
    "alarmTimeagoUrgent" : false,
    "alarmTimeagoUrgentMins" : "30",
    "alarmPumpBatteryLow" : false,
    "language" : "en",
    "scaleY" : "linear",
    "showPlugins" : "careportal sage iob openaps pump basal cob delta direction upbat delta direction upbat",
    "showForecast" : "openaps",
    "focusHours" : 3,
    "heartbeat" : 60,
    "baseURL" : "https://redacted/",
    "authDefaultRoles" : "denied",
    "thresholds" : {
      "bgHigh" : 260,
      "bgTargetTop" : 180,
      "bgTargetBottom" : 72,
      "bgLow" : 55
    },
    "insecureUseHttp" : true,
    "secureHstsHeader" : true,
    "secureHstsHeaderIncludeSubdomains" : false,
    "secureHstsHeaderPreload" : false,
    "secureCsp" : false,
    "deNormalizeDates" : false,
    "showClockDelta" : false,
    "showClockLastTime" : false,
    "frameUrl1" : "",
    "frameUrl2" : "",
    "frameUrl3" : "",
    "frameUrl4" : "",
    "frameUrl5" : "",
    "frameUrl6" : "",
    "frameUrl7" : "",
    "frameUrl8" : "",
    "frameName1" : "",
    "frameName2" : "",
    "frameName3" : "",
    "frameName4" : "",
    "frameName5" : "",
    "frameName6" : "",
    "frameName7" : "",
    "frameName8" : "",
    "authFailDelay" : 5000,
    "adminNotifiesEnabled" : true,
    "authenticationPromptOnLoad" : false,
    "DEFAULT_FEATURES" : [ "bgnow", "delta", "direction", "timeago", "devicestatus", "upbat", "errorcodes", "profile", "bolus", "dbsize", "runtimestate", "basal", "careportal" ],
    "alarmTypes" : [ "simple" ],
    "enable" : [ "careportal", "sage", "iob", "basal", "openaps", "pump", "cob", "rawbg", "cage", "iage", "bage", "treatmentnotify", "bgnow", "delta", "direction", "timeago", "devicestatus", "upbat", "errorcodes", "profile", "cors", "treatmentnotify", "bolus", "dbsize", "runtimestate", "simplealarms" ]
  },
  "extendedSettings" : {
    "pump" : {
      "warnBattP" : 10,
      "urgentBattP" : 5,
      "warnBattV" : 1.23,
      "urgentBattV" : 1.19,
      "fields" : "battery reservoir clock status"
    },
    "openaps" : {
      "colorPredictionLines" : true
    },
    "cage" : {
      "display" : "days",
      "warn" : 336,
      "urgent" : 504
    },
    "sage" : {
      "warn" : 336,
      "urgent" : 504
    },
    "iage" : {
      "warn" : 336,
      "urgent" : 504
    },
    "basal" : {
      "render" : "icicle"
    },
    "bolus" : {
      "renderFormatSmall" : "minimal",
      "renderOver" : 0.8
    },
    "dbsize" : {
      "inMib" : true,
      "max" : 19073.5
    },
    "devicestatus" : {
      "advanced" : true,
      "days" : 1
    }
  },
  "authorized" : null,
  "runtimeState" : "loaded"
}
        "#;

        let status: Status = serde_json::from_str(json).unwrap();
        println!("{:#?}", status);

    }
}