//! A Simple amp model of a Champ Amp
//!

use serde_json::json;
use crate::dsp::biquad::{BiQuadFilter, FilterType};
use crate::dsp::clip::{clip_sample, ClipType};
use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::pedal::Pedal;

/// Simple Champ Amp model
///
/// The amp sim does the following stuff.
/// - Tone stack
/// - give some tube non-linearity
/// - Run the 4 biquad speaker cab sim for the 1x12 amp

pub struct Champ {
    bypass: bool,
    settings: Vec<PedalSetting<f64>>,

    // Volume
    gain: f32,
    master_vol: f32,
    // Tone Controls
    bass_filter: BiQuadFilter,
    treble_filter: BiQuadFilter,
    // Speaker Sim
    high_pass: BiQuadFilter,
    notch: BiQuadFilter,
    high_shelf: BiQuadFilter,
    low_pass: BiQuadFilter,
}

impl Champ {
    /// Construct a new default TemplatePedal
    pub fn new() -> Champ {
        let mut pedal = Champ {
            bypass: false,
            settings: Vec::new(),
            bass_filter: BiQuadFilter::new(),
            treble_filter: BiQuadFilter::new(),
            high_pass: BiQuadFilter::new(),
            notch: BiQuadFilter::new(),
            high_shelf: BiQuadFilter::new(),
            low_pass: BiQuadFilter::new(),
            gain: 1.0,
            master_vol: 1.0,
        };
        pedal.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "gain",
            vec![],
            0.0,
            -6.0,
            20.0,
            0.25,
        ));
        pedal.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "treble",
            vec![],
            0.0,
            -20.0,
            20.0,
            0.25,
        ));
        pedal.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "bass",
            vec![],
            0.0,
            -20.0,
            20.0,
            0.25,
        ));
        pedal.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "master",
            vec![],
            0.0,
            -10.0,
            10.0,
            0.25,
        ));

        pedal
            .bass_filter
            .init(FilterType::LowShelf, 200.0, 1.0, 1.0, 48000.0);
        pedal
            .treble_filter
            .init(FilterType::HighShelf, 2000.0, 1.0, 1.0, 48000.0);

        // Initialize the cab sim filters
        pedal.high_pass
            .init(FilterType::HighPass, 90.0, 1.0, 0.707, 48000.0);
        pedal.notch
            .init(FilterType::Notch, 700.0, -16.0, 2.0, 48000.0);
        pedal.high_shelf
            .init(FilterType::HighShelf, 800.0, 8.0, 0.707, 48000.0);
        pedal.low_pass
            .init(FilterType::LowPass, 4000.0, 1.0, 0.707, 48000.0);

        pedal
    }
}

impl Pedal for Champ {
    /// This is used by the PedalBoard to change a value from the U/X.
    ///
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
    /// Called to reconfigure the effect based on the current setting values.
    ///
    fn load_from_settings(&mut self) -> () {
        for setting in &mut self.settings {
            if setting.dirty {
                match setting.get_name() {
                    "gain" => {
                        self.gain = setting.stype.convert(setting.get_value()) as f32;
                    }
                    "master" => {
                        self.master_vol = setting.stype.convert(setting.get_value()) as f32;
                    }
                    "treble" => {
                        self.treble_filter.init(
                            FilterType::HighShelf,
                            2000.0,
                            setting.get_value(),
                            1.0,
                            48000.0,
                        );
                    }
                    "bass" => {
                        self.bass_filter.init(
                            FilterType::LowShelf,
                            200.0,
                            setting.get_value(),
                            1.0,
                            48000.0,
                        );
                    }
                    _ => (),
                }
                setting.dirty = false;
            }
        }
    }

    /// This function gets called on a frame of audio.  This is where you filter does what it does.
    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> () {
        let mut i: usize = 0;
        for samp in input {
            // Apply tone stacks
            let mut value = self.bass_filter.get_sample(samp);
            value = self.treble_filter.get_sample(&value);
            // Compression
            value = clip_sample(&ClipType::Exp, value * self.gain);
            value /= self.gain;
            value *= self.master_vol;
            // Cabinet sim
            value = self.notch.get_sample(&value);
            value = self.high_shelf.get_sample(&value);
            value = self.low_pass.get_sample(&value);
            value = self.low_pass.get_sample(&value);
            // copy value to output
            output[i] = value;
            i += 1;
        }
    }
    /// returns the bypass setting on the pedal
    /// have to do this here because the Pedal interface cannot have any member data
    fn bypass(&self) -> bool {
        self.bypass
    }
    /// set the bypass on the pedal
    /// have to do this here because the Pedal interface cannot have any member data
    fn set_my_bypass(&mut self, val: bool) -> () {
        self.bypass = val;
    }
    /// Serialize the pedal and settings.  This allows the PedalBoard to save itself and reconstruct
    /// from a saved configuration
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
            "name": "Champ",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_champ {
    use super::*;

    #[test]
    fn can_build() {
        let mut ts = Champ::new();
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
