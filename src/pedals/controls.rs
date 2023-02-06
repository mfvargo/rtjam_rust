use serde_json::{json, Value};

#[derive(ToPrimitive, FromPrimitive)]
pub enum SettingType {
    Float = 0,
    Int,
    Boolean,
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
            setting_type: SettingType::Float,
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
          "type": num::ToPrimitive::to_usize(&self.setting_type),
        })
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
    fn can_json() {
        let setting = PedalSetting::build();
        let j_val = setting.to_json();
        println!("jval: {}", j_val);
        assert_eq!(j_val["name"], setting.name);
    }
}
