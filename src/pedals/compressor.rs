use crate::dsp::peak_detector::PeakDetector;
use crate::utils::{to_db, to_lin};
use serde_json::json;

use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::pedal::Pedal;

pub struct Compressor {
    bypass: bool,
    settings: Vec<PedalSetting<f64>>,
    threshold: f64,
    ratio: f64,
    slope: f64,
    level: f64,
    attack: f64,
    release: f64,
    peak: PeakDetector,
}

impl Compressor {
    pub fn new() -> Compressor {
        let mut comp = Compressor {
            bypass: false,
            settings: Vec::new(),
            threshold: 0.0,
            ratio: 1.0,
            slope: 1.0,
            level: 1.0,
            attack: 0.0,
            release: 0.0,
            peak: PeakDetector::build(0.0, 0.0, 48_000),
        };
        comp.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "threshold",
            vec![],
            -20.0,
            -80.0,
            20.0,
            0.5,
        ));
        comp.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "ratio",
            vec![],
            1.0,
            1.0,
            20.0,
            0.1,
        ));
        comp.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "level",
            vec![],
            0.0,
            -20.0,
            20.0,
            0.5,
        ));

        comp.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Msec,
            "attack",
            vec![],
            20.0,
            2.0,
            200.0,
            0.5,
        ));
        comp.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Msec,
            "release",
            vec![],
            120.0,
            50.0,
            1000.0,
            0.5,
        ));
        comp.load_from_settings();
        comp
    }
}

impl Pedal for Compressor {
    fn bypass(&self) -> bool {
        self.bypass
    }
    fn set_my_bypass(&mut self, val: bool) -> () {
        self.bypass = val;
    }
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
                    "threshold" => {
                        // Note that even though threshold is in db, don't connvert it cause the algorithm
                        // uses it in dB
                        // TODO: maybe we can fix this later
                        self.threshold = setting.get_value();
                    }
                    "ratio" => {
                        self.ratio = setting.stype.convert(setting.get_value());
                    }
                    "level" => {
                        self.level = setting.stype.convert(setting.get_value());
                    }
                    "attack" => {
                        self.attack = setting.stype.convert(setting.get_value());
                    }
                    "release" => {
                        self.release = setting.stype.convert(setting.get_value());
                    }
                    _ => (),
                }
                setting.dirty = false;
            }
        }
        self.slope = 1.0 - (1.0 / self.ratio);
        self.peak.init(self.attack, self.release, 48_000);
    }
    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> () {
        let mut i: usize = 0;
        for samp in input {
            // sample as f64
            let inp = *samp as f64;
            // This part is all in DB
            // convert level to dB (all gain computations are in log space)
            let input_level = to_db(self.peak.get(inp));

            // set gain to 0dB
            let mut compressor_gain = 0.0;
            // if signal is above threshold, calculate new gain based on level
            // above threshold and ratio
            if input_level > self.threshold {
                compressor_gain = self.slope * (self.threshold - input_level);
            }
            // This part is now back to linear
            // convert gain in dB to linear value
            compressor_gain = to_lin(compressor_gain);

            // multiply incoming signal by gain computer output (dynamic)
            // and level (make-up gain)
            output[i] = (inp * compressor_gain * self.level) as f32;
            i += 1;
        }
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
            "name": "Compressor",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_noise_gate {
    use super::*;

    #[test]
    fn can_build() {
        let mut comp = Compressor::new();
        let input: Vec<f32> = vec![0.2, 0.3, 0.4];
        let mut output: Vec<f32> = vec![0.0; 3];
        comp.process(&input, &mut output);
        // Need a way to see if the output is what is should be!
        println!("output: {:?}", output);
        // assert_eq!(output[0], 0.0);
        comp.bypass = true;
        comp.process(&input, &mut output);
        assert_eq!(output[0], 0.2);
        println!("json: {}", comp.as_json(23));
    }
}
