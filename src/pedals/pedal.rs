use serde_json::json;

use super::controls::SettingUnit;

pub trait Pedal {
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

    // These functions are related to bypass
    fn bypass(&self) -> bool;
    fn set_my_bypass(&mut self, val: bool) -> ();
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

    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> ();

    fn as_json(&self, index: usize) -> serde_json::Value;

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

    fn do_change_a_value(&mut self, name: &str, value: &serde_json::Value) -> ();
    fn load_from_settings(&mut self) -> ();
}
