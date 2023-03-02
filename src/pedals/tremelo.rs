use serde_json::json;

use crate::dsp::low_freq_osc::{LowFreqOsc, WaveShape};

use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::pedal::Pedal;

pub struct Tremelo {
    pub bypass: bool,
    settings: Vec<PedalSetting<f32>>,
    osc: LowFreqOsc<f32>,
    depth: f32,
    rate: f32,
}

impl Tremelo {
    pub fn new() -> Tremelo {
        let mut trem = Tremelo {
            bypass: false,
            settings: Vec::new(),
            depth: 0.0,
            rate: 1.0,
            osc: LowFreqOsc::new(),
        };
        trem.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "depth",
            vec![],
            0.4,
            0.0,
            1.0,
            0.05,
        ));
        trem.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "rate",
            vec![],
            1.2,
            0.01,
            8.0,
            0.1,
        ));
        // Initialize from settings
        trem.load_from_settings();
        trem
    }
}

impl Pedal for Tremelo {
    fn do_change_a_value(&mut self, name: &str, val: &serde_json::Value) {
        // Find the setting using the name, then update it's value
        match val.as_f64() {
            Some(f) => {
                for setting in &mut self.settings {
                    if setting.get_name() == name {
                        setting.set_value(f as f32);
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
                    "depth" => {
                        self.depth = setting.get_value();
                    }
                    "rate" => {
                        self.rate = setting.get_value();
                    }
                    _ => (),
                }
                setting.dirty = false;
            }
        }
        self.osc
            .init(WaveShape::Sine, self.rate, self.depth, 48_000.0);
    }

    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> () {
        let mut i: usize = 0;
        for samp in input {
            output[i] = (1.0 + self.osc.get_sample()) * samp;
            i += 1;
        }
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
            "name": "Tremelo",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_tonestack {
    use super::*;

    #[test]
    fn can_build() {
        let mut di = Tremelo::new();
        let input: Vec<f32> = vec![0.2, 0.3, 0.4];
        let mut output: Vec<f32> = vec![0.0; 3];
        di.process(&input, &mut output);
        // Need a way to see if the output is what is should be!
        println!("output: {:?}", output);
        // assert_eq!(output[0], 0.0);
        di.bypass = true;
        di.process(&input, &mut output);
        assert_eq!(output[0], 0.2);
        println!("json: {}", di.as_json(23));
    }
}
