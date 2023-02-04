//! Structure used to pass messages to the audio engine from the websocket

use serde_json::json;
use simple_error::bail;
use std::fmt;

use crate::common::box_error::BoxError;

pub struct ParamMessage {
    pub param: i64,
    pub ivalue_1: i64,
    pub ivalue_2: i64,
    pub fvalue: f64,
    pub svalue: String,
}

impl ParamMessage {
    pub fn new(param: i64, ival1: i64, ival2: i64, fval: f64, sval: &str) -> ParamMessage {
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
          "param": self.param,
          "iValue1": self.ivalue_1,
          "iValue2": self.ivalue_2,
          "fValue": self.fvalue,
          "sValue": self.svalue,
        })
    }
    pub fn from_string(data: &str) -> Result<ParamMessage, BoxError> {
        let raw = serde_json::from_str(data)?;
        dbg!(&raw);
        Self::from_json(&raw)
    }
    pub fn from_json(raw: &serde_json::Value) -> Result<ParamMessage, BoxError> {
        if !(raw["param"].is_i64() || raw["param"].is_string()) {
            bail!("no param in message");
        }
        let mut msg = ParamMessage::new(0, 0, 0, 0.0, "");
        if raw["param"].is_i64() {
            msg.param = raw["param"].as_i64().unwrap();
        }
        if raw["param"].is_string() {
            msg.param = str::parse(raw["param"].as_str().unwrap())?;
        }
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
}

impl fmt::Display for ParamMessage {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ param: {}, ival_1: {}, ival_2: {}, fval: {} sval: {} }}",
            self.param, self.ivalue_1, self.ivalue_2, self.fvalue, self.svalue
        )
    }
}
#[cfg(test)]
mod test_param_message {
    use super::*;

    #[test]
    fn can_json() {
        let msg = ParamMessage::new(0, 1, 2, 3.0, "bob");
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

    #[test]
    fn from_json_string_4() {
        let data = "{\"param\":1006,\"iValue1\":150}";
        let msg = ParamMessage::from_string(data).unwrap();
        assert_eq!(msg.param, 1006);
    }
}

// TODO:  convert this into rust for the param
// might need some help from https://enodev.fr/posts/rusticity-convert-an-integer-to-an-enum.html#:~:text=Converting%20an%20integer%20to%20an%20enum%20in%20Rust,Cargo.toml%2C%20add%20dependencies%20for%20num%2C%20num-derive%2C%20and%20num-traits%3A
