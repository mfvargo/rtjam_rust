//! Defines the trait for a Pedal.  
//!
//! By implementing this trait the PedalBoard can
//! have lots of different types of effect pedals without needing the details of
//! each pedals implementation.  A good exammple of an Effect implementing this
//! trait is the [`crate::pedals::tone_stack::ToneStack`]
use serde_json::json;

use super::controls::SettingUnit;

/// trait that Effects (aka Pedals) must define to allow them to be in a [`crate::pedals::pedal_board::PedalBoard`]
pub trait Pedal {
    /// called by the PedalBoard to have a frame of audio processed.
    ///
    /// The bypass functionality is common to all pedals so it's implemented in
    /// this process function.  If the bypass is on, the function just copies the
    /// input data to the output.  If bypass is off, the do_algorithm function is called
    /// on the Effect that is implementing this trait.
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> () {
        if self.bypass() {
            let mut i: usize = 0;
            for samp in input {
                output[i] = *samp;
                i += 1;
            }
        } else {
            self.do_algorithm(input, output);
        }
    }
    /// this returns a control setting for the bypass switch on the pedal.
    ///
    /// Effects with the Pedal trait can use this to supply their bypass setting along
    /// with their specific settings.
    fn make_bypass(&self) -> serde_json::Value {
        json!({
            "index": 0,
            "labels": [],
            "max": 1,
            "min": 0,
            "name": "bypass",
            "step": 1,
            "type": num::ToPrimitive::to_usize(&SettingUnit::Footswitch),
            "value": self.bypass(),
        })
    }
    /// Effects with this trait must tell if they are bypassed or not
    fn bypass(&self) -> bool;
    /// Top down function to set change the bypass value of the effect.
    fn set_my_bypass(&mut self, val: bool) -> ();
    /// Effects must implement this function to process a frame of audio data.
    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> ();
    /// Effects must implment this to serialize their settings.  They can
    /// use the provided make bypass function as the first value in their settings.
    fn as_json(&self, index: usize) -> serde_json::Value;
    /// Top down call from the PedalBoard.  It is used by the U/X to change the
    /// setting on a Pedal.  The default implementation will process the bypass
    /// setting (common to all pedals).  If the setting is not "bypass", it will call
    /// do_change_a_value() and load_from_settings() on the Effect.
    fn change_setting(&mut self, setting: &serde_json::Value) -> () {
        if let Some(v) = setting["name"].as_str() {
            if let Some(b) = setting["value"].as_bool() {
                self.set_my_bypass(b);
            } else {
                self.do_change_a_value(v, &setting["value"]);
                self.load_from_settings();
            }
        }
    }
    /// Effects must implement this.  This is how their settings are changed (knobs are turned)
    fn do_change_a_value(&mut self, name: &str, value: &serde_json::Value) -> ();
    /// This is called by the framework so that pedals can re-calculate their internal values
    /// after settings have been changed.
    fn load_from_settings(&mut self) -> ();
}
