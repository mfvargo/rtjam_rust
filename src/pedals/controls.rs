use crate::utils::to_lin;
use serde::Serialize;
use serde_json::json;

#[derive(ToPrimitive, FromPrimitive)]
pub enum SettingUnit {
    Continuous,
    Selector,
    Footswitch,
}

pub enum SettingType {
    Msec,
    DB,
    Linear,
}

impl SettingType {
    pub fn convert(&self, value: f64) -> f64 {
        match self {
            Self::DB => to_lin(value),
            Self::Msec => value / 1000.0,
            _ => value,
        }
    }
}

pub struct PedalSetting<T> {
    pub stype: SettingType,
    units: SettingUnit,
    name: String,
    labels: Vec<String>,
    value: T,
    min: T,
    max: T,
    step: T,
}

impl<T: PartialOrd + Copy + Serialize> PedalSetting<T> {
    pub fn new(
        units: SettingUnit,
        stype: SettingType,
        name: &str,
        labels: Vec<String>,
        value: T,
        min: T,
        max: T,
        step: T,
    ) -> PedalSetting<T> {
        // Make sure min is not greater than max
        let mut min_val = min;
        if min_val > max {
            min_val = max;
        }
        // create a new thing
        let mut thg = PedalSetting {
            units: units,
            stype: stype,
            name: String::from(name),
            labels: labels,
            value: min,
            min: min_val,
            max: max,
            step: step,
        };
        thg.set_value(value);
        thg
    }
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn set_value(&mut self, value: T) {
        if value < self.min {
            self.value = self.min;
        } else if value > self.max {
            self.value = self.max;
        } else {
            self.value = value;
        }
    }

    pub fn get_value(&self) -> T {
        self.value
    }

    pub fn as_json(&self, idx: usize) -> serde_json::Value {
        json!({
            "index": idx,
            "name": self.name,
            "labels": self.labels,
            "min": self.min,
            "max": self.max,
            "step": self.step,
            "value": self.value,
            "type": num::ToPrimitive::to_usize(&self.units),
        })
    }
}

#[cfg(test)]

mod test_pedal_setttings {
    use super::*;

    #[test]
    fn sthing() {
        let mut f = PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "gain",
            vec![],
            -10.0,
            -60.0,
            100.0,
            0.5,
        );
        assert_eq!(f.get_value(), -10.0);
        f.set_value(-120.0);
        assert_eq!(f.get_value(), -60.0);
        println!("json: {}", f.as_json(33));
    }

    fn build_a_db(val: f64) -> PedalSetting<f64> {
        PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "gain",
            vec![],
            val,
            -60.0,
            100.0,
            0.5,
        )
    }

    #[test]
    fn convert() {
        let setting = build_a_db(-10.0);
        assert_eq!(setting.stype.convert(setting.get_value()), 0.1)
    }
    #[test]
    fn can_set() {}

    #[test]
    fn can_json_out() {
        let setting = build_a_db(-10.0);
        let j_val = setting.as_json(1);
        println!("jval: {}", j_val);
        assert_eq!(j_val["name"], setting.name);
        assert_eq!(j_val["value"], -10.0)
    }
}
