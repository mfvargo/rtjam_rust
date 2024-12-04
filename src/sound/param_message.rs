//! Structure used to pass messages to the audio engine from the websocket

use num::{FromPrimitive, ToPrimitive};
use serde_json::json;
use simple_error::bail;
use std::fmt;

use crate::common::box_error::BoxError;

#[derive(FromPrimitive, ToPrimitive)]
pub enum JamParam {
    ChanGain1 = 0,  //deprecated
    ChanGain2,  //deprecated
    ChanGain3,  //deprecated
    ChanGain4,  //deprecated
    ChanGain5,  //deprecated
    ChanGain6,  //deprecated
    ChanGain7,  //deprecated
    ChanGain8,  //deprecated
    ChanGain9,  //deprecated
    ChanGain10,  //deprecated
    ChanGain11,  //deprecated
    ChanGain12,  //deprecated
    ChanGain13,  //deprecated
    ChanGain14,  //deprecated
    MasterVol,  // Set gain on master volume
    InputMonitor,  //deprecated
    Room0,  //deprecated
    Room1,  //deprecated
    Room2,  //deprecated
    ReverbChanOne,  //deprecated
    ReverbMix,  //deprecated
    RoomChange, // Connect to a room (aka joinRoom)
    Disconnect, // Disconnect from a room
    HPFOn,  //deprecated
    HPFOff,  //deprecated
    ReverbOne,  //deprecated
    ReverbTwo,  //deprecated
    GetConfigJson,
    SetEffectConfig,
    InsertPedal,
    DeletePedal,
    MovePedal,
    LoadBoard,
    TuneChannel,
    MetronomeVolume,  // TBD
    SetFader,  // Change fader on a channel
    MuteToRoom, // Send silence to the room (but not to the mixer)
    ConnectionKeepAlive,
    SetBufferSize,  // Changes the framesize (experimental for Joel, did not help)
    ChannelMute,  // Set the must stus on a channel
    ChannelGain,  // Set the channel gain (replaces ChanGain1-14)
    Count,  // Count of basic apis  (not really used)
    SetAudioInput = 1000,
    SetAudioOutput,
    ListAudioConfig,
    CheckForUpdate,
    RandomCommand,
    GetPedalTypes,
    SetUpdateInterval,  // Sets the frequency the unit will update the ux in the browser
    RebootDevice = 9998,  // deprecated
    ShutdownDevice = 9999,
    StopAudio,    // Stop the jamEngine audio component
}

/// The ParamMessage is used to define the API to the sound engine from the outside
/// world.  Interpretation of this message is by the
/// [`JamEngine`](crate::sound::jam_engine::JamEngine).
///
/// A ParamMessage consist of a param value [`JamParam`], and 4
/// other values.  interpretation of the values is dependent on the nature of the command.
///
/// other values are ivalue_1: integer, ivalue_2: integer, fvalue: float, and svalue: string.
///
/// ### TODO
/// This encoding needs to get normalized.  But it will require coordination between the u/x and the
/// sound unit.
pub struct ParamMessage {
    pub param: JamParam,
    pub ivalue_1: i64,
    pub ivalue_2: i64,
    pub fvalue: f64,
    pub svalue: String,
}

impl ParamMessage {
    pub fn new(param: JamParam, ival1: i64, ival2: i64, fval: f64, sval: &str) -> ParamMessage {
        ParamMessage {
            param: param,
            ivalue_1: ival1,
            ivalue_2: ival2,
            fvalue: fval,
            svalue: String::from(sval),
        }
    }
    pub fn as_json(&self) -> serde_json::Value {
        json!({
          "param": num::ToPrimitive::to_usize(&self.param),
          "iValue1": self.ivalue_1,
          "iValue2": self.ivalue_2,
          "fValue": self.fvalue,
          "sValue": self.svalue,
        })
    }
    pub fn from_string(data: &str) -> Result<ParamMessage, BoxError> {
        let raw = serde_json::from_str(data)?;
        Self::from_json(&raw)
    }
    pub fn from_json(raw: &serde_json::Value) -> Result<ParamMessage, BoxError> {
        if !(raw["param"].is_i64() || raw["param"].is_string()) {
            bail!("no param in message");
        }
        let mut param: Option<JamParam> = None;
        if raw["param"].is_i64() {
            param = FromPrimitive::from_i64(raw["param"].as_i64().unwrap());
        }
        if raw["param"].is_string() {
            param = FromPrimitive::from_i64(str::parse(raw["param"].as_str().unwrap())?);
        }
        match param {
            Some(p) => {
                let mut msg = ParamMessage::new(p, 0, 0, 0.0, "");
                if raw["iValue1"].is_i64() {
                    msg.ivalue_1 = raw["iValue1"].as_i64().unwrap();
                }
                if raw["iValue1"].is_string() {
                    msg.ivalue_1 = str::parse(raw["iValue1"].as_str().unwrap())?;
                }
                if raw["iValue2"].is_i64() {
                    msg.ivalue_2 = raw["iValue2"].as_i64().unwrap();
                }
                if raw["iValue2"].is_string() {
                    msg.ivalue_2 = str::parse(raw["iValue2"].as_str().unwrap())?;
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

impl fmt::Display for ParamMessage {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ param: {}, ival_1: {}, ival_2: {}, fval: {} sval: {} }}",
            ToPrimitive::to_i64(&self.param).unwrap(),
            self.ivalue_1,
            self.ivalue_2,
            self.fvalue,
            self.svalue
        )
    }
}
#[cfg(test)]
mod test_param_message {
    use super::*;

    #[test]
    fn can_json() {
        let msg = ParamMessage::new(JamParam::ChanGain11, 1, 2, 3.0, "bob");
        assert!(msg.ivalue_1 == 1);
    }
    #[test]
    fn from_json_string_1() {
        let data = r#"
        {
            "param": 21,
            "iValue1": 1,
            "iValue2": 100,
            "fValue": 2.0,
            "sValue": "John Doe"
        }"#;
        let msg = ParamMessage::from_string(data).unwrap();
        assert_eq!(msg.ivalue_2, 100);
    }
    #[test]
    fn from_json_string_2() {
        let data = r#"
      {
          "param": 21,
          "iValue1": "1",
          "iValue2": "100",
          "fValue": 2.0,
          "sValue": "John Doe"
      }"#;
        let raw: serde_json::Value = serde_json::from_str(data).unwrap();
        let msg = ParamMessage::from_json(&raw).unwrap();
        assert_eq!(msg.ivalue_2, 100);
    }
    #[test]
    fn from_json_string_3() {
        let data = r#"
    {
        "param": 21,
        "iValue1": "1",
        "iValue2": "100",
        "fValue": 2,
        "sValue": "John Doe"
    }"#;
        let raw: serde_json::Value = serde_json::from_str(data).unwrap();
        let msg = ParamMessage::from_json(&raw).unwrap();
        assert_eq!(msg.fvalue, 2.0);
    }
}

// TODO:  convert this into rust for the param
// might need some help from https://enodev.fr/posts/rusticity-convert-an-integer-to-an-enum.html#:~:text=Converting%20an%20integer%20to%20an%20enum%20in%20Rust,Cargo.toml%2C%20add%20dependencies%20for%20num%2C%20num-derive%2C%20and%20num-traits%3A
