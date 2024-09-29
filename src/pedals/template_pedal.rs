//! A blank pedal to be used as a template for others.
//!
//! This template pedal demonstrates the features you must implement to create a new pedal type.

use serde_json::json;
use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::pedal::Pedal;

/// Effect That I will create
///
/// The effect I am making here does the following stuff.
/// - this stuff
/// - that stuff
/// - some other stuff
pub struct TemplatePedal {
    bypass: bool,
    settings: Vec<PedalSetting<f64>>,
    // Here is where you would add th things you need to make it work
    // BiQuad filter, delays, attack_hold_release, see things in the dsp folder.
}

impl TemplatePedal {
    /// Construct a new default TemplatePedal
    pub fn new() -> TemplatePedal {
        let mut pedal = TemplatePedal {
            bypass: false,
            settings: Vec::new(),
        };
        pedal.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "knob 1",
            vec![],
            0.0,
            -20.0,
            20.0,
            0.25,
        ));
        pedal.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "knob 2",
            vec![],
            0.0,
            -20.0,
            20.0,
            0.25,
        ));
        pedal.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "knob 3",
            vec![],
            0.0,
            -20.0,
            20.0,
            0.25,
        ));
        // Initialize the the things you might use.
        pedal
    }
}

impl Pedal for TemplatePedal {
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
                    // This is where you apply the particular settings to this pedal
                    // How you apply really depends on what the pedal is doing and how you 
                    // want to interpret the settings.
                    // Commented out values below are just examples (see ToneStack)
                    // "treble" => {
                    //     self.treble_filter.init(
                    //         FilterType::HighShelf,
                    //         2000.0,
                    //         setting.get_value(),
                    //         1.0,
                    //         48000.0,
                    //     );
                    // }
                    // "mid" => {
                    //     self.mid_filter.init(
                    //         FilterType::Peaking,
                    //         700.0,
                    //         setting.get_value(),
                    //         2.0,
                    //         48000.0,
                    //     );
                    // }
                    // "bass" => {
                    //     self.bass_filter.init(
                    //         FilterType::LowShelf,
                    //         200.0,
                    //         setting.get_value(),
                    //         1.0,
                    //         48000.0,
                    //     );
                    // }
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
            // For this template, I am just going to copy input to output
            output[i] = *samp;
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
            "name": "Template Pedal",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_template_pedal {
    use super::*;

    #[test]
    fn can_build() {
        let mut ts = TemplatePedal::new();
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
