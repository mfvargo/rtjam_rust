use crate::utils::to_db;
use serde_json::{json, Value};

pub enum SettingType {
    Float(f64),
    Int(i64),
    Boolean(bool),
}

#[derive(ToPrimitive, FromPrimitive)]
pub enum SettingUnit {
    Msec = 0,
    DB,
    Linear,
    Selector,
    Footswitch,
}

pub struct PedalSetting {
    name: String,
    labels: Vec<String>,
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    units: SettingUnit,
    setting_type: SettingType,
}

impl PedalSetting {
    pub fn build(name: &str, labels: Vec<String>, value: f64) -> PedalSetting {
        PedalSetting {
            name: String::from(name),
            labels: labels,
            value: value,
            min: -100.0,
            max: 100.0,
            step: 1.0,
            units: SettingUnit::Linear,
            setting_type: SettingType::Float(0.0),
        }
    }
    pub fn to_json(&self, idx: usize) -> Value {
        json!({
          "index": idx,
          "labels": self.labels,
          "name": self.name,
          "value": self.value,
          "min": self.min,
          "max": self.max,
          "step": self.step,
          "units": num::ToPrimitive::to_usize(&self.units),
          "type": self.setting_type_to_json(),
        })
    }
    fn setting_type_to_json(&self) -> serde_json::Value {
        match self.setting_type {
            SettingType::Float(f) => serde_json::Value::from(match self.units {
                SettingUnit::Msec => f / 1000.0,
                SettingUnit::DB => to_db(f) as f64,
                SettingUnit::Linear => f,
                SettingUnit::Selector => f,
                SettingUnit::Footswitch => f,
            }),
            SettingType::Int(i) => serde_json::Value::from(i),
            SettingType::Boolean(b) => serde_json::Value::from(b),
        }
    }
    pub fn get_value_float(&self) -> Option<f64> {
        match self.setting_type {
            SettingType::Float(f) => Some(f),
            _ => None,
        }
    }
    pub fn is_true(&self) -> Option<bool> {
        match self.setting_type {
            SettingType::Boolean(b) => Some(b),
            _ => None,
        }
    }
    pub fn get_integer_value(&self) -> Option<i64> {
        match self.setting_type {
            SettingType::Int(i) => Some(i),
            _ => None,
        }
    }
}

#[cfg(test)]

mod test_pedal_setttings {
    use super::*;

    fn build_a_db() -> PedalSetting {
        PedalSetting::build("gain", vec![], -12.0)
    }

    #[test]
    fn can_build() {
        let setting = build_a_db();
        assert_eq!(setting.value, -12.0);
    }

    #[test]
    fn can_json_out() {
        let setting = build_a_db();
        let j_val = setting.to_json(1);
        println!("jval: {}", j_val);
        assert_eq!(j_val["name"], setting.name);
    }
    #[test]
    fn can_json_in() {
        let data = r#"
        {
            "index": 0,
            "labels": [],
            "max": 1,
            "min": 0,
            "name": "bypass",
            "step": 1,
            "type": 2,
            "units": 4,
            "value": false
          }
        "#;
        let jval: Value = serde_json::from_str(data).unwrap();
        println!("json: {}", jval);
    }
}
