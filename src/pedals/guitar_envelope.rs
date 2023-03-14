//! Envelope tracking pedal (auto-wah) Implemented using the [`EnvelopeBase`] module.  
//!
//! has specific features for guitar
//! TODO:  Figure out if there are any, otherwise just use one pedal for
//! both bass and guitar

use num::FromPrimitive;
use serde_json::json;

use crate::dsp::biquad::FilterType;

use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::envelope_base::EnvelopeBase;
use super::pedal::Pedal;

pub struct GuitarEnvelope {
    pub bypass: bool,
    settings: Vec<PedalSetting<f64>>,
    env_mode: PedalSetting<i64>,
    env_base: EnvelopeBase,
}

#[derive(ToPrimitive, FromPrimitive)]
pub enum EnvelopeMode {
    LPF,
    BPF,
    HPF,
}

impl GuitarEnvelope {
    pub fn new() -> GuitarEnvelope {
        let mut env = GuitarEnvelope {
            bypass: false,
            settings: Vec::new(),
            env_mode: PedalSetting::new(
                SettingUnit::Selector,
                SettingType::Linear,
                "type",
                vec![
                    String::from("LPF"),
                    String::from("BPF"),
                    String::from("HPF"),
                ],
                num::ToPrimitive::to_i64(&EnvelopeMode::LPF).unwrap(),
                num::ToPrimitive::to_i64(&EnvelopeMode::LPF).unwrap(),
                num::ToPrimitive::to_i64(&EnvelopeMode::HPF).unwrap(),
                1,
            ),
            env_base: EnvelopeBase::new(),
        };
        env.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Msec,
            "frequency",
            vec![],
            20.0,
            1.0,
            2000.0,
            5.0,
        ));
        env.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "resonance",
            vec![],
            4.0,
            1.0,
            20.0,
            0.05,
        ));
        env.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "sensitivity",
            vec![],
            50.0,
            0.0,
            100.0,
            1.0,
        ));
        env.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Msec,
            "attack",
            vec![],
            20.0,
            2.0,
            100.0,
            0.5,
        ));
        env.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Msec,
            "release",
            vec![],
            250.0,
            10.0,
            1000.0,
            2.0,
        ));
        env.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "level",
            vec![],
            0.0,
            -10.0,
            10.0,
            0.1,
        ));
        env.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "dry level",
            vec![],
            -80.0,
            -80.0,
            0.0,
            0.1,
        ));
        env
    }
}

impl Pedal for GuitarEnvelope {
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
                if name == "type" {
                    self.env_mode.set_value(i);
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
                    "frequency" => {
                        self.env_base.freq = setting.get_value();
                    }
                    "resonance" => {
                        self.env_base.resonance = setting.get_value();
                    }
                    "sensitivity" => {
                        self.env_base.sensitivity = setting.get_value();
                    }
                    "attack" => {
                        self.env_base.attack = setting.stype.convert(setting.get_value());
                    }
                    "release" => {
                        self.env_base.release = setting.stype.convert(setting.get_value());
                    }
                    "level" => {
                        self.env_base.level = setting.stype.convert(setting.get_value());
                    }
                    "dry level" => {
                        self.env_base.dry = setting.stype.convert(setting.get_value());
                    }
                    _ => (),
                }
                setting.dirty = false;
            }
        }
        if self.env_mode.dirty {
            if let Some(mode) = FromPrimitive::from_i64(self.env_mode.get_value()) {
                match mode {
                    EnvelopeMode::LPF => {
                        self.env_base.ftype = FilterType::LowPass;
                    }
                    EnvelopeMode::BPF => {
                        self.env_base.ftype = FilterType::BandPass;
                    }
                    EnvelopeMode::HPF => {
                        self.env_base.ftype = FilterType::HighPass;
                    }
                }
            }
        }
        self.env_base.init();
    }

    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> () {
        self.env_base.process(input, output);
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
        settings.push(self.env_mode.as_json(i));
        for item in &self.settings {
            i += 1;
            settings.push(item.as_json(i));
        }
        json!({
            "index": idx,
            "name": "Guitar Envelope",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_delay_pedal {
    use super::*;

    #[test]
    fn can_build() {
        let mut env = GuitarEnvelope::new();
        println!("base: {}", env.env_base);
        let input: Vec<f32> = vec![0.2, 0.3, 0.4];
        let mut output: Vec<f32> = vec![0.0; 3];
        env.process(&input, &mut output);
        // // Need a way to see if the output is what is should be!
        // println!("output: {:?}", output);
        // // assert_eq!(output[0], 0.0);
        // ts.bypass = true;
        // ts.process(&input, &mut output);
        // assert_eq!(output[0], 0.2);
        println!(
            "json {}",
            serde_json::to_string_pretty(&env.as_json(23)).unwrap()
        );
    }
}
