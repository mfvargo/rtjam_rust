//! Bass DI pedal simulator.  Goal is to approximate the tone stack and
//! compression style of an Ampeg bass head.
//!
//! The pedal consists of 3 tone controls and a level control (aka "Volume")
//! It is implemented with 3 BiQuad filters and the "soft" clip algorithm.
//!
//! All settings are passed in via JSON and units are dB
use serde_json::json;

use crate::dsp::biquad::{BiQuadFilter, FilterType};
use crate::dsp::clip::{clip_sample, ClipType};

use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::pedal::Pedal;

pub struct BassDI {
    pub bypass: bool,
    settings: Vec<PedalSetting<f64>>,
    bass_filter: BiQuadFilter,
    mid_filter: BiQuadFilter,
    treble_filter: BiQuadFilter,
    gain: f32,
}

impl BassDI {
    pub fn new() -> BassDI {
        let mut di_box = BassDI {
            bypass: false,
            settings: Vec::new(),
            bass_filter: BiQuadFilter::new(),
            mid_filter: BiQuadFilter::new(),
            treble_filter: BiQuadFilter::new(),
            gain: 1.0,
        };
        di_box.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "Volume",
            vec![],
            0.0,
            -10.0,
            10.0,
            0.25,
        ));
        di_box.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "Bass",
            vec![],
            0.0,
            -35.0,
            35.0,
            0.25,
        ));
        di_box.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "Mid",
            vec![],
            0.0,
            -35.0,
            35.0,
            0.25,
        ));
        di_box.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "Treble",
            vec![],
            0.0,
            -35.0,
            35.0,
            0.25,
        ));
        // Initialize from settings
        di_box.load_from_settings();
        di_box
    }
}

impl Pedal for BassDI {
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
                    "Volume" => {
                        self.gain = setting.stype.convert(setting.get_value()) as f32;
                    }
                    "Treble" => {
                        self.treble_filter.init(
                            FilterType::HighShelf,
                            350.0,
                            setting.get_value(),
                            0.707,
                            48000.0,
                        );
                    }
                    "Mid" => {
                        self.mid_filter.init(
                            FilterType::Peaking,
                            180.0,
                            setting.get_value(),
                            0.707,
                            48000.0,
                        );
                    }
                    "Bass" => {
                        self.bass_filter.init(
                            FilterType::LowShelf,
                            70.0,
                            setting.get_value(),
                            0.707,
                            48000.0,
                        );
                    }
                    _ => (),
                }
                setting.dirty = false;
            }
        }
    }

    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> () {
        let mut i: usize = 0;
        for samp in input {
            let mut value = self.bass_filter.get_sample(samp);
            value = self.mid_filter.get_sample(&value);
            value = self.treble_filter.get_sample(&value);
            value = clip_sample(&ClipType::Soft, value);
            output[i] = self.gain * value;
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
        let mut i = settings.len();
        for item in &self.settings {
            settings.push(item.as_json(i));
            i += 1;
        }
        json!({
            "index": idx,
            "name": "Bass DI",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_tonestack {
    use super::*;

    #[test]
    fn can_build() {
        let mut di = BassDI::new();
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
