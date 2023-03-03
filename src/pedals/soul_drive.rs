use serde_json::json;

use crate::dsp::clip::ClipType;

use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::distortion_base::DistortionBase;
use super::pedal::Pedal;

pub struct SoulDrive {
    pub bypass: bool,
    settings: Vec<PedalSetting<f64>>,
    dist_base: DistortionBase,
}

impl SoulDrive {
    pub fn new() -> SoulDrive {
        let mut dist = SoulDrive {
            bypass: false,
            settings: Vec::new(),
            dist_base: DistortionBase::new(),
        };
        dist.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "drive",
            vec![],
            9.0,
            0.0,
            60.0,
            0.5,
        ));
        dist.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "treble",
            vec![],
            25.0,
            -10.0,
            30.0,
            0.5,
        ));
        dist.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "level",
            vec![],
            -4.0,
            -60.0,
            12.0,
            0.5,
        ));
        dist.load_from_settings();
        dist
    }
}

impl Pedal for SoulDrive {
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
        for setting in &mut self.settings {
            if setting.dirty {
                match setting.get_name() {
                    "drive" => {
                        self.dist_base.gain1 = setting.stype.convert(setting.get_value()) as f32;
                    }
                    "treble" => {
                        self.dist_base.tone_treble_cut_boost = setting.get_value();
                    }
                    "level" => {
                        self.dist_base.level = setting.stype.convert(setting.get_value()) as f32;
                    }
                    _ => (),
                }
                setting.dirty = false;
            }
        }

        // Read the settings from the map and apply them to our copy of the data.
        self.dist_base.num_stages = 1;
        self.dist_base.hpf1_freq = 120.0;
        self.dist_base.lpf1_freq = 7500.0;
        self.dist_base.clip1_type = ClipType::Hard;
        self.dist_base.hpf2_freq = 55.0;
        self.dist_base.lpf2_freq = 10000.0;
        self.dist_base.hpf3_freq = 110.0;
        self.dist_base.lpf3_freq = 8700.0;
        self.dist_base.tone_bass_cut_boost = 3.0;
        self.dist_base.tone_bass_freq = 150.0;
        self.dist_base.tone_mid_cut_boost = 12.0;
        self.dist_base.tone_mid_q = 1.0;
        self.dist_base.tone_mid_freq = 630.0;
        self.dist_base.tone_treble_freq = 1000.0;
        self.dist_base.dry_level = 0.2;
        self.dist_base.init();
    }

    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> () {
        self.dist_base.process(input, output);
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
            "name": "SoulDrive",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_tonestack {
    use super::*;

    #[test]
    fn can_build() {
        let mut ts = SoulDrive::new();
        let input: Vec<f32> = vec![0.2, 0.3, 0.4];
        let mut output: Vec<f32> = vec![0.0; 3];
        ts.process(&input, &mut output);
        // // Need a way to see if the output is what is should be!
        // println!("output: {:?}", output);
        // // assert_eq!(output[0], 0.0);
        // ts.bypass = true;
        // ts.process(&input, &mut output);
        // assert_eq!(output[0], 0.2);
        println!(
            "json {}",
            serde_json::to_string_pretty(&ts.as_json(23)).unwrap()
        );
    }
}
