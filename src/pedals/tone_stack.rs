use serde_json::json;

use super::controls::{PedalSetting, SettingUnit};
use super::pedal::Pedal;

pub struct ToneStack {
    pub bypass: bool,
    settings: Vec<PedalSetting<f64>>,
}

impl ToneStack {
    pub fn new() -> ToneStack {
        let mut stack = ToneStack {
            bypass: false,
            settings: Vec::new(),
        };
        stack.settings.push(PedalSetting::new(
            SettingUnit::DB,
            "treble",
            vec![],
            0.0,
            -20.0,
            20.0,
            0.25,
        ));
        stack.settings.push(PedalSetting::new(
            SettingUnit::DB,
            "treble",
            vec![],
            0.0,
            -20.0,
            20.0,
            0.25,
        ));
        stack.settings.push(PedalSetting::new(
            SettingUnit::DB,
            "treble",
            vec![],
            0.0,
            -20.0,
            20.0,
            0.25,
        ));
        stack
    }
}

impl Pedal for ToneStack {
    fn do_algorithm(&self, _input: &[f32], _output: &mut [f32]) -> () {
        // Implement algorithm
        println!("Tonestack do_algorithm");
    }
    fn bypass(&self) -> bool {
        self.bypass
    }
    fn as_json(&self, idx: usize) -> serde_json::Value {
        let mut settings: Vec<serde_json::Value> = vec![self.make_bypass()];
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

mod test_pedal_pedal {
    use super::*;

    #[test]
    fn can_build() {
        let mut ts = ToneStack::new();
        let input: Vec<f32> = vec![0.2, 0.3, 0.4];
        let mut output: Vec<f32> = vec![0.0; 3];
        ts.process(&input, &mut output);
        assert_eq!(output[0], 0.0);
        ts.bypass = true;
        ts.process(&input, &mut output);
        assert_eq!(output[0], 0.2);
        println!("json: {}", ts.as_json(23));
    }
}
