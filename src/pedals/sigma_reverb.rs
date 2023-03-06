use serde_json::json;

use crate::dsp::allpass_delay::AllpassDelay;
use crate::dsp::biquad::{BiQuadFilter, FilterType};

use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::pedal::Pedal;

pub struct SigmaReverb {
    pub bypass: bool,
    settings: Vec<PedalSetting<f64>>,
    lf_filter: BiQuadFilter,
    hf_filter: BiQuadFilter,
    delay1: [f32; 4853],
    delay1_idx: usize,
    delay2: [f32; 5888],
    delay2_idx: usize,
    reverb_time: f32,
    reverb_level: f32,
    lap1: AllpassDelay<f32>,
    lap2: AllpassDelay<f32>,
    lap3: AllpassDelay<f32>,
    lap4: AllpassDelay<f32>,
    apd1: AllpassDelay<f32>,
    apd1b: AllpassDelay<f32>,
    apd2: AllpassDelay<f32>,
    apd2b: AllpassDelay<f32>,
}

impl SigmaReverb {
    pub fn new() -> SigmaReverb {
        let mut reverb = SigmaReverb {
            bypass: false,
            settings: Vec::new(),
            lf_filter: BiQuadFilter::new(),
            hf_filter: BiQuadFilter::new(),
            delay1: [0.0; 4853],
            delay1_idx: 0,
            delay2: [0.0; 5888],
            delay2_idx: 0,
            reverb_time: 0.1,
            reverb_level: 0.0,
            lap1: AllpassDelay::new(),
            lap2: AllpassDelay::new(),
            lap3: AllpassDelay::new(),
            lap4: AllpassDelay::new(),
            apd1: AllpassDelay::new(),
            apd1b: AllpassDelay::new(),
            apd2: AllpassDelay::new(),
            apd2b: AllpassDelay::new(),
        };
        // Set up the delaylines
        reverb.lap1.init(229, 0.6);
        reverb.lap2.init(321, 0.6);
        reverb.lap3.init(471, 0.6);
        reverb.lap4.init(803, 0.6);
        reverb.apd1.init(1821, 0.6);
        reverb.apd1b.init(2565, 0.6);
        reverb.apd2.init(2114, 0.6);
        reverb.apd2b.init(1817, 0.6);
        // config the params
        reverb.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "level",
            vec![],
            0.0,
            0.0,
            1.0,
            0.01,
        ));
        reverb.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "time",
            vec![],
            0.01,
            0.0,
            1.0,
            0.01,
        ));
        reverb.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "lowFreq",
            vec![],
            10.0,
            10.0,
            250.0,
            5.0,
        ));
        reverb.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "highFreq",
            vec![],
            8000.0,
            400.0,
            8000.0,
            5.0,
        ));
        // Initialize
        reverb.load_from_settings();
        reverb
    }
}

impl Pedal for SigmaReverb {
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
                    "level" => {
                        self.reverb_level = setting.get_value() as f32;
                    }
                    "time" => {
                        self.reverb_time = setting.get_value() as f32;
                    }
                    "lowFreq" => {
                        self.lf_filter.init(
                            FilterType::HighPass,
                            setting.get_value(),
                            1.0,
                            1.0,
                            48000.0,
                        );
                    }
                    "highFreq" => {
                        self.hf_filter.init(
                            FilterType::LowPass,
                            setting.get_value(),
                            1.0,
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

    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> () {
        let mut i: usize = 0;
        for samp in input {
            let reverb_in = 1.0 * samp; // scale by .2 to match FV-1 impementations

            // read samples from end of each delay line
            let delay1_out = self.delay1[self.delay1_idx];
            let delay2_out = self.delay2[self.delay2_idx];

            // TODO - commets/mods for test only
            // process first allpass chain - lap1->lap4
            let mut value = self.lap1.get_sample(reverb_in);
            value = self.lap2.get_sample(value);
            value = self.lap3.get_sample(value);
            value = self.lap4.get_sample(value);

            let sum1_out = value + (delay2_out * self.reverb_time);

            // process stretched all-pass Chain 2 - AP1->AP1B
            value = self.apd1.get_sample(sum1_out);
            value = self.apd1b.get_sample(value);

            // write result to delay line 1
            self.delay1[self.delay1_idx] = value;
            self.delay1_idx += 1;
            self.delay1_idx %= 4853;

            // read from end of delay line 1
            value = delay1_out * self.reverb_time;

            // process streched all-pass Chain 3 - AP2->AP2B
            // input to Chain 3 is delay 1 output * reverb time
            value = self.apd2.get_sample(value);
            value = self.apd2b.get_sample(value);

            // TODO - add lpf/hpf here
            value = self.lf_filter.get_sample(&value);
            value = self.hf_filter.get_sample(&value);

            // write result to delay line 2
            self.delay2[self.delay2_idx] = value;
            self.delay2_idx += 1;
            self.delay2_idx %= 5888;

            // reverb output = sum of two delay taps
            value = delay1_out + delay2_out;

            // add in reverb to dry signal
            output[i] = samp + value * self.reverb_level;
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
        let mut i = 1;
        for item in &self.settings {
            settings.push(item.as_json(i));
            i += 1;
        }
        json!({
            "index": idx,
            "name": "Sigma Reverb",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_sigma_reverb {
    use super::*;

    #[test]
    fn can_build() {
        let mut reverb = SigmaReverb::new();
        let input: Vec<f32> = vec![0.2, 0.3, 0.4];
        let mut output: Vec<f32> = vec![0.0; 3];
        reverb.process(&input, &mut output);
        // Need a way to see if the output is what is should be!
        println!("output: {:?}", output);
        // assert_eq!(output[0], 0.0);
        reverb.bypass = true;
        reverb.process(&input, &mut output);
        assert_eq!(output[0], 0.2);
        println!("json: {}", reverb.as_json(23));
    }
}
