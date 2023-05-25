use num::{FromPrimitive, ToPrimitive};
use serde_json::json;
use simple_error::bail;
use std::fmt;

use crate::common::box_error::BoxError;

#[derive(FromPrimitive, ToPrimitive, PartialEq, Debug)]
pub enum RoomParam {
    GetTempo = 0,
    SetTempo,
    Record,
    Pause,
    Stop,
    Play,
    ListFiles,
    SaveRecording,
    DeleteRecording,
    UploadRecording,
}
/// The RoomCommandMessage is used to define the API to the room component from the outside
/// world.
///
/// A RoomCommandMessage consist of a param value [`RoomParam`](RoomParam), and 3
/// other values.  interpretation of the values is dependent on the nature of the command.
///
/// other values are ivalue_1: integer, fvalue: float, and svalue: string.
///
/// ### TODO
/// This encoding needs to get normalized.  But it will require coordination between the u/x and the
/// sound unit.
#[derive(Debug)]
pub struct RoomCommandMessage {
    pub param: RoomParam,
    pub ivalue_1: i64,
    pub fvalue: f64,
    pub svalue: String,
}
impl RoomCommandMessage {
    pub fn new(param: RoomParam, ival1: i64, fval: f64, sval: &str) -> RoomCommandMessage {
        RoomCommandMessage {
            param: param,
            ivalue_1: ival1,
            fvalue: fval,
            svalue: String::from(sval),
        }
    }
    pub fn as_json(&self) -> serde_json::Value {
        json!({
          "param": num::ToPrimitive::to_usize(&self.param),
          "iValue1": self.ivalue_1,
          "fValue": self.fvalue,
          "sValue": self.svalue,
        })
    }
    pub fn from_string(data: &str) -> Result<RoomCommandMessage, BoxError> {
        let raw = serde_json::from_str(data)?;
        Self::from_json(&raw)
    }
    pub fn from_json(raw: &serde_json::Value) -> Result<RoomCommandMessage, BoxError> {
        if !(raw["param"].is_i64() || raw["param"].is_string()) {
            bail!("no param in message");
        }
        let mut param: Option<RoomParam> = None;
        if raw["param"].is_i64() {
            param = FromPrimitive::from_i64(raw["param"].as_i64().unwrap());
        }
        if raw["param"].is_string() {
            param = FromPrimitive::from_i64(str::parse(raw["param"].as_str().unwrap())?);
        }
        match param {
            Some(p) => {
                let mut msg = RoomCommandMessage::new(p, 0, 0.0, "");
                if raw["iValue1"].is_i64() {
                    msg.ivalue_1 = raw["iValue1"].as_i64().unwrap();
                }
                if raw["iValue1"].is_string() {
                    msg.ivalue_1 = str::parse(raw["iValue1"].as_str().unwrap())?;
                }
                if raw["fValue"].is_f64() {
                    msg.fvalue = raw["fValue"].as_f64().unwrap();
                }
                if raw["fValue"].is_i64() {
                    msg.fvalue = raw["fValue"].as_i64().unwrap() as f64;
                }
                if raw["fValue"].is_string() {
                    msg.fvalue = str::parse(raw["fValue"].as_str().unwrap())?;
                }
                if raw["sValue"].is_string() {
                    msg.svalue = String::from(raw["sValue"].as_str().unwrap());
                }
                Ok(msg)
            }
            None => {
                bail!("can't extract param");
            }
        }
    }
}

impl fmt::Display for RoomCommandMessage {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ param: {}, ival_1: {}, fval: {} sval: {} }}",
            ToPrimitive::to_i64(&self.param).unwrap(),
            self.ivalue_1,
            self.fvalue,
            self.svalue
        )
    }
}

#[cfg(test)]
mod test_cmd_message {
    use super::*;

    #[test]
    fn can_json() {
        let msg = RoomCommandMessage::new(RoomParam::GetTempo, 21, 3.0, "bob");
        assert!(msg.ivalue_1 == 21);
    }
    #[test]
    fn from_json_string_1() {
        let data = r#"
        {
            "param": 0,
            "iValue1": 1,
            "fValue": 2.0,
            "sValue": "John Doe"
        }"#;
        let msg = RoomCommandMessage::from_string(data).unwrap();
        assert_eq!(msg.fvalue, 2.0);
    }
    #[test]
    fn from_json_string_2() {
        let data = r#"
      {
          "param": 3,
          "iValue1": "1",
          "fValue": 2.0,
          "sValue": "John Doe"
      }"#;
        let raw: serde_json::Value = serde_json::from_str(data).unwrap();
        let msg = RoomCommandMessage::from_json(&raw).unwrap();
        assert_eq!(msg.param, RoomParam::Pause);
    }
}
