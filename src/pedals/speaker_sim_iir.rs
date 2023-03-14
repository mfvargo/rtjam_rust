//! IIR based model to model Speaker cabinets.  Darius secret filter points
use num::FromPrimitive;
use serde_json::json;

use crate::dsp::biquad::{BiQuadFilter, FilterType};

use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::pedal::Pedal;

#[derive(ToPrimitive, FromPrimitive)]
enum CabinetType {
    OneByTwelve,
    TwoByTwelve,
    FourByTen,
    FourByTwelve,
}
pub struct SpeakerSimIIR {
    pub bypass: bool,
    cabinet: PedalSetting<i64>,
    level: PedalSetting<f64>,
    gain: f32,
    high_pass: BiQuadFilter,
    notch: BiQuadFilter,
    high_shelf: BiQuadFilter,
    low_pass: BiQuadFilter,
}

impl SpeakerSimIIR {
    pub fn new() -> SpeakerSimIIR {
        let mut cab = SpeakerSimIIR {
            bypass: false,
            cabinet: PedalSetting::new(
                SettingUnit::Selector,
                SettingType::Linear,
                "cabinetType",
                vec![
                    String::from("1x12"),
                    String::from("2x12"),
                    String::from("4x10"),
                    String::from("4x12"),
                ],
                0,
                num::ToPrimitive::to_i64(&CabinetType::OneByTwelve).unwrap(),
                num::ToPrimitive::to_i64(&CabinetType::FourByTwelve).unwrap(),
                1,
            ),
            level: PedalSetting::new(
                SettingUnit::Continuous,
                SettingType::DB,
                "Level",
                vec![],
                0.0,
                -20.0,
                20.0,
                0.25,
            ),
            gain: 1.0,
            high_pass: BiQuadFilter::new(),
            notch: BiQuadFilter::new(),
            high_shelf: BiQuadFilter::new(),
            low_pass: BiQuadFilter::new(),
        };
        cab.load_from_settings();
        cab
    }
}

impl Pedal for SpeakerSimIIR {
    fn do_change_a_value(&mut self, name: &str, val: &serde_json::Value) {
        // Find the setting using the name, then update it's value
        match name {
            "Level" => {
                if let Some(l) = val.as_f64() {
                    self.level.set_value(l);
                }
            }
            "cabinetType" => {
                if let Some(i) = val.as_i64() {
                    self.cabinet.set_value(i);
                }
            }
            _ => {}
        };
    }
    fn load_from_settings(&mut self) -> () {
        // change my member variables based on the settings
        if self.level.dirty {
            self.gain = self.level.stype.convert(self.level.get_value()) as f32;
            self.level.dirty = false;
        }
        if self.cabinet.dirty {
            match FromPrimitive::from_i64(self.cabinet.get_value()) {
                Some(CabinetType::OneByTwelve) => {
                    self.high_pass
                        .init(FilterType::HighPass, 90.0, 1.0, 0.707, 48000.0);
                    self.notch
                        .init(FilterType::Notch, 700.0, -16.0, 2.0, 48000.0);
                    self.high_shelf
                        .init(FilterType::HighShelf, 800.0, 8.0, 0.707, 48000.0);
                    self.low_pass
                        .init(FilterType::LowPass, 4000.0, 1.0, 0.707, 48000.0);
                }
                Some(CabinetType::TwoByTwelve) => {
                    self.high_pass
                        .init(FilterType::HighPass, 90.0, 1.0, 0.707, 48000.0);
                    self.notch
                        .init(FilterType::Notch, 550.0, -16.0, 2.0, 48000.0);
                    self.high_shelf
                        .init(FilterType::HighShelf, 700.0, 8.0, 0.707, 48000.0);
                    self.low_pass
                        .init(FilterType::LowPass, 4000.0, 1.0, 0.707, 48000.0);
                }
                Some(CabinetType::FourByTen) => {
                    self.high_pass
                        .init(FilterType::HighPass, 70.0, 1.0, 0.707, 48000.0);
                    self.notch
                        .init(FilterType::Notch, 400.0, -16.0, 2.0, 48000.0);
                    self.high_shelf
                        .init(FilterType::HighShelf, 400.0, 8.0, 0.707, 48000.0);
                    self.low_pass
                        .init(FilterType::LowPass, 4000.0, 1.0, 0.707, 48000.0);
                }
                Some(CabinetType::FourByTwelve) => {
                    self.high_pass
                        .init(FilterType::HighPass, 40.0, 1.0, 0.707, 48000.0);
                    self.notch
                        .init(FilterType::Notch, 300.0, -12.0, 2.0, 48000.0);
                    self.high_shelf
                        .init(FilterType::HighShelf, 500.0, 5.0, 0.707, 48000.0);
                    self.low_pass
                        .init(FilterType::LowPass, 4200.0, 1.0, 0.707, 48000.0);
                }
                None => {}
            }
            self.cabinet.dirty = false;
        }
    }

    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> () {
        let mut i = 0;
        for samp in input {
            let mut value = self.high_pass.get_sample(samp);
            value = self.notch.get_sample(&value);
            value = self.high_shelf.get_sample(&value);
            value = self.low_pass.get_sample(&value);
            value = self.low_pass.get_sample(&value);
            output[i] = value * self.gain;
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
        settings.push(self.level.as_json(1));
        settings.push(self.cabinet.as_json(2));
        json!({
            "index": idx,
            "name": "Speaker Sim",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_tonestack {
    use super::*;

    #[test]
    fn can_build() {
        let mut ts = SpeakerSimIIR::new();
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
