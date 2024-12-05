//! A pedal to model a room with a few echos
//!
//! This room modeler will have a number of surfaces with colors

use log::debug;
use num::FromPrimitive;
use serde_json::json;
use crate::dsp::biquad::{BiQuadFilter, FilterType};
use crate::dsp::delay_line::DelayLine;

use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::pedal::Pedal;

/// Room Simulator
///
/// The more details to come
/// - this stuff
/// - that stuff
/// - some other stuff
/// 

#[derive(ToPrimitive, FromPrimitive)]
enum SurfaceType {
    Hard,
    Firm,
    Soft,
    Mush,
}

impl SurfaceType {
    pub fn labels() -> Vec<String> {
        vec![
                String::from("Hard"),
                String::from("Firm"),
                String::from("Soft"),
                String::from("Mush"),
            ]
    }
    pub fn setting(name: &str) -> PedalSetting<i64> {
        PedalSetting::new(
            SettingUnit::Selector,
            SettingType::Linear,
            name,
            SurfaceType::labels(),
            0,
            num::ToPrimitive::to_i64(&SurfaceType::Hard).unwrap(),
            num::ToPrimitive::to_i64(&SurfaceType::Mush).unwrap(),
            1,
        )
    }
    pub fn gain_from_type(v: i64, filt: &mut BiQuadFilter) -> f32 {
        match FromPrimitive::from_i64(v) {
            Some(SurfaceType::Hard) => {
                filt.init(
                    FilterType::LowPass,
                    1000.0,
                    1.0,
                    1.0,
                    48000.0,
                );
                0.9
            }
            Some(SurfaceType::Firm) => {
                filt.init(
                    FilterType::LowPass,
                    700.0,
                    1.0,
                    1.0,
                    48000.0,
                );
                0.7
            }
            Some(SurfaceType::Soft) => {
                filt.init(
                    FilterType::LowPass,
                    300.0,
                    1.0,
                    1.0,
                    48000.0,
                );
                0.4
            }
            Some(SurfaceType::Mush) => {
                filt.init(
                    FilterType::LowPass,
                    100.0,
                    1.0,
                    1.0,
                    48000.0,
                );
                0.1
            }
            None => {
                filt.init(
                    FilterType::LowPass,
                    4000.0,
                    1.0,
                    1.0,
                    48000.0,
                );
                0.0
            }
        }
    }
}
pub struct RoomSimulator {
    bypass: bool,
    delay_settings: Vec<PedalSetting<f64>>,
    surface_settings: Vec<PedalSetting<i64>>,
    // Here is where you would add th things you need to make it work
    // BiQuad filter, delays, attack_hold_release, see things in the dsp folder.
    side: DelayLine<f32>,
    side_filter: BiQuadFilter,
    floor: DelayLine<f32>,
    floor_filter: BiQuadFilter,
    top: DelayLine<f32>,
    top_filter: BiQuadFilter
}

impl RoomSimulator {
    /// Construct a new default TemplatePedal
    pub fn new() -> RoomSimulator {
        let mut pedal = RoomSimulator {
            bypass: false,
            delay_settings: Vec::new(),
            surface_settings: Vec::new(),
            side: DelayLine::new(),
            side_filter: BiQuadFilter::new(),
            floor: DelayLine::new(),
            floor_filter: BiQuadFilter::new(),
            top: DelayLine::new(),
            top_filter: BiQuadFilter::new(),
        };
        pedal.delay_settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "side",
            vec![],
            10.0,
            1.0,
            20.0,
            0.25,
        ));
        pedal.surface_settings.push(SurfaceType::setting("sideSurface"));
        pedal.delay_settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "floor",
            vec![],
            10.0,
            1.0,
            20.0,
            0.25,
        ));
        pedal.surface_settings.push(SurfaceType::setting("floorSurface"));
        pedal.delay_settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "top",
            vec![],
            10.0,
            6.0,
            30.0,
            0.25,
        ));
        pedal.surface_settings.push(SurfaceType::setting("topSurface"));
        // Initialize the the things you might use.
        pedal.load_from_settings();
        // pedal.side.init(10 * 48, 0.9);
        // pedal.floor.init(10 * 48, 0.9);
        // pedal.top.init(10 * 48, 0.9);
        pedal
    }
}

impl Pedal for RoomSimulator {
    /// This is used by the PedalBoard to change a value from the U/X.
    ///
    fn do_change_a_value(&mut self, name: &str, val: &serde_json::Value) {
        debug!("changing: {} to {}", name, val);
        // Find the setting using the name, then update it's value
        match val.as_f64() {
            Some(f) => {
                for setting in &mut self.delay_settings {
                    if setting.get_name() == name {
                        debug!("doing the change f32: {}", f);
                        setting.set_value(f);
                    }
                }
            }
            _ => ()
        }
        match val.as_i64() {
            Some(i) => {
                for setting in &mut self.surface_settings {
                    if setting.get_name() == name {
                        debug!("doing the change i64: {}", i);
                        setting.set_value(i);
                    }
                }
            }
            _ => ()
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
        for setting in &mut self.delay_settings {
            if setting.dirty {
                let new_length = (setting.get_value() * 48.0) as usize;
                match setting.get_name() {
                    "side" => {
                        self.side.set_length(new_length);
                    }
                    "floor" => {
                        self.floor.set_length(new_length);
                    }
                    "top" => {
                        self.top.set_length(new_length);
                    }
                    _ => (),
                }
                setting.dirty = false;
            }
        }
        for setting in &mut self.surface_settings {
            if setting.dirty {
                match setting.get_name() {
                    "sideSurface" => {
                        self.side.set_gain(SurfaceType::gain_from_type(setting.get_value(), &mut self.side_filter));
                    }
                    "floorSurface" => {
                        self.side.set_gain(SurfaceType::gain_from_type(setting.get_value(), &mut self.floor_filter));
                    }
                    "topSurface" => {
                        self.side.set_gain(SurfaceType::gain_from_type(setting.get_value(), &mut self.top_filter));
                    }
                    _ => (),
                }
                setting.dirty = false;
            }
        }
    }

    /// This function gets called on a frame of audio.  This is where you filter does what it does.
    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> () {
        for (i, samp) in input.iter().enumerate() {
            // For this template, I am just going to copy input to output
            let mut agg = *samp;  // direct pass through
            agg += self.side_filter.get_sample(&self.side.get_sample(*samp)); // side delay filtered
            agg += self.floor_filter.get_sample(&self.floor.get_sample(*samp)); // floor delay filtered
            agg += self.top_filter.get_sample(&self.top.get_sample(*samp)); // top delay filtered
        
            // agg += self.side.get_sample(*samp); // side delay filtered
            // agg += self.floor.get_sample(*samp); // floor delay filtered
            // agg += self.top.get_sample(*samp); // top delay filtered

            output[i] = agg;
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
        for j in [0, 1, 2] {
            settings.push(self.delay_settings[j].as_json(i));
            i += 1;
            settings.push(self.surface_settings[j].as_json(i));
            i += 1;
        }
        json!({
            "index": idx,
            "name": "Room Simulator",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_template_pedal {
    use super::*;

    #[test]
    fn can_build() {
        let mut ts = RoomSimulator::new();
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
