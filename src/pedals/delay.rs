//! Delay pedal.   Implemented using the [`DelayBase`] module.
//!
//! It also has some drift features.  It is a skin on delay_base
use num::FromPrimitive;
use serde_json::json;

use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::delay_base::{DelayBase, DelayMode};
use super::pedal::Pedal;

pub struct Delay {
    pub bypass: bool,
    settings: Vec<PedalSetting<f64>>,
    delay_mode: PedalSetting<i64>,
    delay_base: DelayBase,
}

impl Delay {
    pub fn new() -> Delay {
        let mut delay = Delay {
            bypass: false,
            settings: Vec::new(),
            delay_mode: PedalSetting::new(
                SettingUnit::Selector,
                SettingType::Linear,
                "delayMode",
                vec![
                    String::from("Dig"),
                    String::from("Ana"),
                    String::from("HPF"),
                ],
                num::ToPrimitive::to_i64(&DelayMode::Digital).unwrap(),
                num::ToPrimitive::to_i64(&DelayMode::Digital).unwrap(),
                num::ToPrimitive::to_i64(&DelayMode::HighPass).unwrap(),
                1,
            ),
            delay_base: DelayBase::new(),
        };
        delay.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Msec,
            "duration",
            vec![],
            250.0,
            2.0,
            500.0,
            1.0,
        ));
        delay.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "feedback",
            vec![],
            0.1,
            0.0,
            1.0,
            0.01,
        ));
        delay.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "level",
            vec![],
            0.5,
            0.0,
            1.0,
            0.01,
        ));
        delay.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "drift",
            vec![],
            -42.0,
            -60.0,
            -25.0,
            1.0,
        ));
        delay.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "rate",
            vec![],
            1.4,
            0.1,
            5.0,
            0.1,
        ));
        delay
    }
}

impl Pedal for Delay {
    fn do_change_a_value(&mut self, name: &str, val: &serde_json::Value) {
        // Find the setting using the name, then update it's value
        match val.as_f64() {
            Some(f) => {
                for setting in &mut self.settings {
                    if setting.get_name() == name {
                        setting.set_value(f);
                    }
                }
            }
            _ => (),
        }
        match val.as_i64() {
            Some(i) => {
                if name == "delayMode" {
                    self.delay_mode.set_value(i);
                }
            }
            _ => (),
        }
    }
    fn load_from_settings(&mut self) -> () {
        // change my member variables based on the settings
        for setting in &mut self.settings {
            if setting.dirty {
                match setting.get_name() {
                    "duration" => {
                        self.delay_base.current_delay_time =
                            setting.stype.convert(setting.get_value()) as f32;
                    }
                    "feedback" => {
                        self.delay_base.feedback = setting.get_value() as f32;
                    }
                    "level" => {
                        self.delay_base.level = setting.get_value() as f32;
                    }
                    "drift" => {
                        self.delay_base.drift = setting.stype.convert(setting.get_value()) as f32;
                    }
                    "rate" => {
                        self.delay_base.drift_rate = setting.get_value() as f32;
                    }
                    _ => (),
                }
                setting.dirty = false;
            }
        }
        if self.delay_mode.dirty {
            self.delay_base.delay_mode =
                FromPrimitive::from_i64(self.delay_mode.get_value()).unwrap();
        }
        self.delay_base.init();
    }

    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> () {
        self.delay_base.process(input, output);
    }
    fn bypass(&self) -> bool {
        self.bypass
    }
    fn set_my_bypass(&mut self, val: bool) -> () {
        self.bypass = val;
    }
    fn as_json(&self, idx: usize) -> serde_json::Value {
        // pass in the bypass setting
        let mut settings: Vec<serde_json::Value> = vec![self.make_bypass()];
        // now the actual settings
        let mut i = 1;
        for item in &self.settings {
            settings.push(item.as_json(i));
            i += 1;
        }
        settings.push(self.delay_mode.as_json(i));
        json!({
            "index": idx,
            "name": "Delay",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_delay_pedal {
    use super::*;

    #[test]
    fn can_build() {
        let mut ts = Delay::new();
        println!("base: {}", ts.delay_base);
        let input: Vec<f32> = vec![0.2, 0.3, 0.4];
        let mut output: Vec<f32> = vec![0.0; 3];
        ts.process(&input, &mut output);
        // // Need a way to see if the output is what is should be!
        // println!("output: {:?}", output);
        // // assert_eq!(output[0], 0.0);
        // ts.bypass = true;
        // ts.process(&input, &mut output);
        // assert_eq!(output[0], 0.2);
        println!(
            "json {}",
            serde_json::to_string_pretty(&ts.as_json(23)).unwrap()
        );
    }
}
