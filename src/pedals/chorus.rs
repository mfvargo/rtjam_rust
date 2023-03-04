use serde_json::json;

use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::delay_base::{DelayBase, DelayMode};
use super::pedal::Pedal;

pub struct Chorus {
    pub bypass: bool,
    settings: Vec<PedalSetting<f64>>,
    delay_base: DelayBase,
}

impl Chorus {
    pub fn new() -> Chorus {
        let mut delay = Chorus {
            bypass: false,
            settings: Vec::new(),
            delay_base: DelayBase::new(),
        };
        delay.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Msec,
            "duration",
            vec![],
            10.0,
            5.0,
            20.0,
            0.05,
        ));
        delay.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "depth",
            vec![],
            0.5,
            0.2,
            0.99,
            0.01,
        ));
        delay.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "drift",
            vec![],
            -12.0,
            -25.0,
            -6.0,
            0.5,
        ));
        delay.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "rate",
            vec![],
            2.0,
            0.1,
            5.0,
            0.1,
        ));
        delay.load_from_settings();
        delay
    }
}

impl Pedal for Chorus {
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
                    "depth" => {
                        let depth = setting.get_value() as f32;
                        self.delay_base.level = depth;
                        self.delay_base.gain = 1.0 - (depth / 2.0);
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
        self.delay_base.feedback = 0.00;
        self.delay_base.delay_mode = DelayMode::HighPass;
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
        json!({
            "index": idx,
            "name": "Chorus",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_delay_pedal {
    use super::*;

    #[test]
    fn can_build() {
        let mut chorus = Chorus::new();
        println!("base: {}", chorus.delay_base);
        let input: Vec<f32> = vec![0.2, 0.3, 0.4];
        let mut output: Vec<f32> = vec![0.0; 3];
        chorus.process(&input, &mut output);
        // // Need a way to see if the output is what is should be!
        // println!("output: {:?}", output);
        // // assert_eq!(output[0], 0.0);
        // ts.bypass = true;
        // ts.process(&input, &mut output);
        // assert_eq!(output[0], 0.2);
        println!(
            "json {}",
            serde_json::to_string_pretty(&chorus.as_json(23)).unwrap()
        );
    }
}
