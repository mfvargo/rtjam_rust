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
    index: usize,
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    units: SettingUnit,
    setting_type: SettingType,
}

impl PedalSetting {
    pub fn build() -> PedalSetting {
        PedalSetting {
            name: String::from("empty"),
            labels: vec![],
            index: 0,
            value: 0.0,
            min: -100.0,
            max: 100.0,
            step: 1.0,
            units: SettingUnit::Linear,
            setting_type: SettingType::Float(0.0),
        }
    }
    pub fn to_json(&self) -> Value {
        json!({
          "index": self.index,
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
    fn setting_type_to_json(&self) -> usize {
        match self.setting_type {
            SettingType::Float(_f) => 0,
            SettingType::Int(_i) => 1,
            SettingType::Boolean(_b) => 2,
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

    #[test]
    fn can_build() {
        let setting = PedalSetting::build();
        assert_eq!(setting.index, 0);
    }

    #[test]
    fn can_json_out() {
        let setting = PedalSetting::build();
        let j_val = setting.to_json();
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
