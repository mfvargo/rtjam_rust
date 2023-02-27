use crate::dsp::attack_hold_release::AttackHoldRelease;
use serde_json::json;

use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::pedal::Pedal;

pub struct NoiseGate {
    bypass: bool,
    settings: Vec<PedalSetting<f64>>,
    threshold: f64,
    attack: f64,
    hold: f64,
    release: f64,
    attack_hold_release: AttackHoldRelease,
}

impl NoiseGate {
    pub fn new() -> NoiseGate {
        let mut gate = NoiseGate {
            bypass: false,
            settings: Vec::new(),
            threshold: 0.0,
            attack: 0.0,
            hold: 0.0,
            release: 0.0 / 1000.0,
            attack_hold_release: AttackHoldRelease::new(0.0, 0.0, 0.0, 48_000),
        };
        gate.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "threshold",
            vec![],
            -50.0,
            -70.0,
            -35.0,
            0.25,
        ));
        gate.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Msec,
            "attack",
            vec![],
            10.0,
            2.0,
            100.0,
            1.0,
        ));
        gate.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Msec,
            "hold",
            vec![],
            40.0,
            20.0,
            250.0,
            1.0,
        ));
        gate.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Msec,
            "release",
            vec![],
            100.0,
            10.0,
            450.0,
            1.0,
        ));
        gate.load_from_settings();
        gate
    }
}

impl Pedal for NoiseGate {
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
        for setting in &self.settings {
            match setting.get_name() {
                "threshold" => {
                    self.threshold = setting.stype.convert(setting.get_value());
                }
                "attack" => {
                    self.attack = setting.stype.convert(setting.get_value());
                }
                "hold" => {
                    self.hold = setting.stype.convert(setting.get_value());
                }
                "release" => {
                    self.release = setting.stype.convert(setting.get_value());
                }
                _ => (),
            }
        }
        self.attack_hold_release =
            AttackHoldRelease::new(self.attack, self.hold, self.release, 48_000);
    }
    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> () {
        let mut i: usize = 0;
        for samp in input {
            output[i] = samp
                * self
                    .attack_hold_release
                    .get(samp.abs() > self.threshold as f32);
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
            "name": "Noise Gate",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_noise_gate {
    use super::*;

    #[test]
    fn can_build() {
        let mut ts = NoiseGate::new();
        let input: Vec<f32> = vec![0.2, 0.3, 0.4];
        let mut output: Vec<f32> = vec![0.0; 3];
        ts.process(&input, &mut output);
        // Need a way to see if the output is what is should be!
        println!("output: {:?}", output);
        // assert_eq!(output[0], 0.0);
        ts.bypass = true;
        ts.process(&input, &mut output);
        assert_eq!(output[0], 0.2);
        println!("json: {}", ts.as_json(23));
    }
}
