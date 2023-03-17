//! Effect to simulate the Tone controls on a typical amplifier.
//!
//! It uses three [`BiQuadFilter`](crate::dsp::biquad::BiQuadFilter) filters to implement a tone stack similar
//! to that on a Fender amp.  The cutoff frequencies of the filters and their Q are preset.  The settings
//! here just affect the boost on the filters.
use serde_json::json;

use crate::dsp::biquad::{BiQuadFilter, FilterType};

use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::pedal::Pedal;

/// Effect that looks like a 3 band tone control
///
/// The ToneStack has 3 settings adjustable from -20 to +20 dB (sorry they don't go to 11)
/// - treble
/// - mid
/// - bass
pub struct ToneStack {
    bypass: bool,
    settings: Vec<PedalSetting<f64>>,
    bass_filter: BiQuadFilter,
    mid_filter: BiQuadFilter,
    treble_filter: BiQuadFilter,
}

impl ToneStack {
    /// Construct a new default ToneStack with all three controls at 0dB
    pub fn new() -> ToneStack {
        let mut stack = ToneStack {
            bypass: false,
            settings: Vec::new(),
            bass_filter: BiQuadFilter::new(),
            mid_filter: BiQuadFilter::new(),
            treble_filter: BiQuadFilter::new(),
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
    /// This is used by the PedalBoard to change a value from the U/X.
    ///
    /// Valid settings are Float values corresponding to settings named "trebel", "mid", or "bass"
    /// Settings are in dB -20.0 to +20.0
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
    /// called after a setting change.  Note that settings are marked dirty on
    /// update so the ToneStatck only needs to tweak those things that have changed.
    ///
    /// * A special note, most all settings are linearized when used in the algorithms with the
    /// exception of the Boost parameter of the BiQuadFilter.  This parameter is passed in as
    /// dB.  Normally there would be a call the the settings units to convert them, but in
    /// this case the value is passed directly to the BiQuadFilter.  (great example, huh?)

    fn load_from_settings(&mut self) -> () {
        // change my member variables based on the settings
        for setting in &mut self.settings {
            if setting.dirty {
                match setting.get_name() {
                    "treble" => {
                        self.treble_filter.init(
                            FilterType::HighShelf,
                            2000.0,
                            setting.get_value(),
                            1.0,
                            48000.0,
                        );
                    }
                    "mid" => {
                        self.mid_filter.init(
                            FilterType::Peaking,
                            700.0,
                            setting.get_value(),
                            2.0,
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

    /// Apply the filters to tweak the tones
    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> () {
        let mut i: usize = 0;
        for samp in input {
            let mut value = self.bass_filter.get_sample(samp);
            value = self.mid_filter.get_sample(&value);
            output[i] = self.treble_filter.get_sample(&value);
            i += 1;
        }
    }
    /// returns the bypass setting on the pedal
    fn bypass(&self) -> bool {
        self.bypass
    }
    /// set the bypass on the pedal
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
