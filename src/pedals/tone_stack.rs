use serde_json::json;

use crate::dsp::biquad::{BiQuadFilter, FilterType};

use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::pedal::Pedal;

pub struct ToneStack {
    pub bypass: bool,
    settings: Vec<PedalSetting<f64>>,
    bass_filter: BiQuadFilter,
    mid_filter: BiQuadFilter,
    treble_filter: BiQuadFilter,
    bass_gain: f32,
    mid_gain: f32,
    treble_gain: f32,
}

impl ToneStack {
    pub fn new() -> ToneStack {
        let mut stack = ToneStack {
            bypass: false,
            settings: Vec::new(),
            bass_filter: BiQuadFilter::new(),
            mid_filter: BiQuadFilter::new(),
            treble_filter: BiQuadFilter::new(),
            bass_gain: 1.0,
            mid_gain: 1.0,
            treble_gain: 1.0,
        };
        stack.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "treble",
            vec![],
            0.0,
            -20.0,
            20.0,
            0.25,
        ));
        stack.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "mid",
            vec![],
            0.0,
            -20.0,
            20.0,
            0.25,
        ));
        stack.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "bass",
            vec![],
            0.0,
            -20.0,
            20.0,
            0.25,
        ));
        // Initialize the filters
        stack
            .bass_filter
            .init(FilterType::LowShelf, 200.0, 1.0, 1.0, 48000.0);
        stack
            .mid_filter
            .init(FilterType::Peaking, 700.0, 1.0, 2.0, 48000.0);
        stack
            .treble_filter
            .init(FilterType::HighShelf, 2000.0, 1.0, 1.0, 48000.0);
        stack
    }
}

impl Pedal for ToneStack {
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
                "treble" => {
                    self.treble_gain = setting.stype.convert(setting.get_value()) as f32;
                }
                "mid" => {
                    self.mid_gain = setting.stype.convert(setting.get_value()) as f32;
                }
                "bass" => {
                    self.bass_gain = setting.stype.convert(setting.get_value()) as f32;
                }
                _ => (),
            }
        }
    }

    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> () {
        let mut i: usize = 0;
        for samp in input {
            output[i] = self.bass_gain * self.bass_filter.get_sample(samp);
            output[i] += self.mid_gain * self.mid_filter.get_sample(samp);
            output[i] += self.treble_gain * self.treble_filter.get_sample(samp);
            output[i] /= 3.0;
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
            "name": "Tone Stack",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_tonestack {
    use super::*;

    #[test]
    fn can_build() {
        let mut ts = ToneStack::new();
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
