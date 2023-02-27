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

    fn bypass(&self) -> bool {
        false
    }

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

    fn change_setting(&mut self, setting: serde_json::Value) -> () {
        match setting["name"].as_str() {
            Some(v) => match v {
                "bypass" => match setting["value"].as_bool() {
                    Some(b) => {
                        self.set_my_bypass(b);
                    }
                    None => (),
                },
                _ => {
                    self.do_change_a_value(v, &setting["value"]);
                    self.load_from_settings();
                }
            },
            None => (),
        }
    }

    fn do_change_a_value(&mut self, name: &str, value: &serde_json::Value) -> ();
    fn load_from_settings(&mut self) -> ();
}
