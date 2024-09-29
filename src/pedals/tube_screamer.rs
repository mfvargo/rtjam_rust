//! A blank pedal to be used as a template for others.
//!
//! This template pedal demonstrates the features you must implement to create a new pedal type.
#![allow(unused_parens)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(non_upper_case_globals)]

use serde_json::json;
use super::controls::{PedalSetting, SettingType, SettingUnit};
use super::pedal::Pedal;

/// Effect That I will create
///
/// The effect I am making here does the following stuff.
/// - this stuff
/// - that stuff
/// - some other stuff

// Faustie crap
type F32 = f32;
type F64 = f64;

fn mydsp_faustpower2_f(value: F32) -> F32 {
	return value * value;
}
fn mydsp_faustpower3_f(value: F32) -> F32 {
	return value * value * value;
}


pub struct TubeScreamer {
    bypass: bool,
    settings: Vec<PedalSetting<f64>>,
    // Variables from the Faust model
    fSampleRate: i32,
	fConst0: F32,
	fConst1: F32,
	fConst2: F32,
	fHslider0: F32,
	fRec0: [F32;2],
	fHslider1: F32,
	fRec2: [F32;2],
	fConst3: F32,
	fConst4: F32,
	fConst5: F32,
	fConst6: F32,
	fConst7: F32,
	fConst8: F32,
	fConst9: F32,
	fHslider2: F32,
	fRec4: [F32;2],
	fVec0: [F32;2],
	fConst10: F32,
	fConst11: F32,
	fConst12: F32,
	fRec5: [F32;2],
	fRec3: [F32;3],
	fVec1: [F32;2],
	fConst13: F32,
	fRec1: [F32;2],
}

impl TubeScreamer {
    /// Construct a new default TubeScreamer
    pub fn new() -> TubeScreamer {
        let mut pedal = TubeScreamer {
            bypass: false,
            settings: Vec::new(),
            fSampleRate: 0,
			fConst0: 0.0,
			fConst1: 0.0,
			fConst2: 0.0,
			fHslider0: 0.0,
			fRec0: [0.0;2],
			fHslider1: 0.0,
			fRec2: [0.0;2],
			fConst3: 0.0,
			fConst4: 0.0,
			fConst5: 0.0,
			fConst6: 0.0,
			fConst7: 0.0,
			fConst8: 0.0,
			fConst9: 0.0,
			fHslider2: 0.0,
			fRec4: [0.0;2],
			fVec0: [0.0;2],
			fConst10: 0.0,
			fConst11: 0.0,
			fConst12: 0.0,
			fRec5: [0.0;2],
			fRec3: [0.0;3],
			fVec1: [0.0;2],
			fConst13: 0.0,
			fRec1: [0.0;2],
        };
        pedal.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "drive",
            vec![],
            25.0,
            1.0,
            120.0,
            0.05,
        ));
        pedal.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "tone",
            vec![],
            0.28,
            0.0,
            1.0,
            0.01,
        ));
        pedal.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "output",
            vec![],
            1.0,
            0.0,
            2.0,
            0.01,
        ));
        // Initialize the the things you might use.
        pedal.instance_init(48000);
        pedal
    }
    // Faust initialization code
    fn instance_reset_params(&mut self) {
		self.fHslider0 = 1.0;
		self.fHslider1 = 0.28;
		self.fHslider2 = 25.0;
	}
	fn instance_clear(&mut self) {
		for l0 in 0..2 {
			self.fRec0[l0 as usize] = 0.0;
		}
		for l1 in 0..2 {
			self.fRec2[l1 as usize] = 0.0;
		}
		for l2 in 0..2 {
			self.fRec4[l2 as usize] = 0.0;
		}
		for l3 in 0..2 {
			self.fVec0[l3 as usize] = 0.0;
		}
		for l4 in 0..2 {
			self.fRec5[l4 as usize] = 0.0;
		}
		for l5 in 0..3 {
			self.fRec3[l5 as usize] = 0.0;
		}
		for l6 in 0..2 {
			self.fVec1[l6 as usize] = 0.0;
		}
		for l7 in 0..2 {
			self.fRec1[l7 as usize] = 0.0;
		}
	}
	fn instance_constants(&mut self, sample_rate: i32) {
		self.fSampleRate = sample_rate;
		self.fConst0 = F32::min(1.92e+05, F32::max(1.0, (self.fSampleRate) as F32));
		self.fConst1 = 44.1 / self.fConst0;
		self.fConst2 = 1.0 - self.fConst1;
		self.fConst3 = 3.1415927 / self.fConst0;
		self.fConst4 = F32::tan(2513.2742 / self.fConst0);
		self.fConst5 = 2.0 * (1.0 - 1.0 / mydsp_faustpower2_f(self.fConst4));
		self.fConst6 = 1.0 / self.fConst4;
		self.fConst7 = (self.fConst6 + -2.0) / self.fConst4 + 1.0;
		self.fConst8 = (self.fConst6 + 2.0) / self.fConst4 + 1.0;
		self.fConst9 = 1.0 / self.fConst8;
		self.fConst10 = 1.0 / F32::tan(2261.9468 / self.fConst0);
		self.fConst11 = 1.0 - self.fConst10;
		self.fConst12 = 1.0 / (self.fConst10 + 1.0);
		self.fConst13 = 0.5 / self.fConst8;
	}
	fn instance_init(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate);
		self.instance_reset_params();
		self.instance_clear();
	}

    // Compute function from Faust
    fn compute(&mut self, count: i32, inputs: &[&[f32]], outputs: &mut[&mut[f32]]) {
		let (inputs0) = if let [inputs0, ..] = inputs {
			let inputs0 = inputs0[..count as usize].iter();
			(inputs0)
		} else {
			panic!("wrong number of inputs");
		};
		let (outputs0) = if let [outputs0, ..] = outputs {
			let outputs0 = outputs0[..count as usize].iter_mut();
			(outputs0)
		} else {
			panic!("wrong number of outputs");
		};
		let mut fSlow0: F32 = self.fConst1 * self.fHslider0;
		let mut fSlow1: F32 = self.fConst1 * self.fHslider1;
		let mut fSlow2: F32 = self.fConst1 * self.fHslider2;
		let zipped_iterators = inputs0.zip(outputs0);
		for (input0, output0) in zipped_iterators {
			self.fRec0[0] = fSlow0 + self.fConst2 * self.fRec0[1];
			self.fRec2[0] = fSlow1 + self.fConst2 * self.fRec2[1];
			let mut fTemp0: F32 = 1.0 / F32::tan(self.fConst3 * (7.25e+03 * self.fRec2[0] + 3e+02));
			self.fRec4[0] = fSlow2 + self.fConst2 * self.fRec4[1];
			let mut fTemp1: F32 = *input0;
			self.fVec0[0] = fTemp1;
			self.fRec5[0] = -(self.fConst12 * (self.fConst11 * self.fRec5[1] - self.fConst10 * (fTemp1 - self.fVec0[1])));
			let mut fTemp2: F32 = self.fRec5[0] * self.fRec4[0];
			self.fRec3[0] = fTemp2 / (F32::abs(fTemp2) + 1.0) - self.fConst9 * (self.fConst7 * self.fRec3[2] + self.fConst5 * self.fRec3[1]);
			let mut fTemp3: F32 = self.fRec3[2] + self.fRec3[0] + 2.0 * self.fRec3[1];
			self.fVec1[0] = fTemp3;
			self.fRec1[0] = (self.fConst13 * (fTemp3 + self.fVec1[1]) - self.fRec1[1] * (1.0 - fTemp0)) / (fTemp0 + 1.0);
			*output0 = self.fRec1[0] * F32::powf(1e+01, 2.0 * (self.fRec0[0] + -1.0));
			self.fRec0[1] = self.fRec0[0];
			self.fRec2[1] = self.fRec2[0];
			self.fRec4[1] = self.fRec4[0];
			self.fVec0[1] = self.fVec0[0];
			self.fRec5[1] = self.fRec5[0];
			self.fRec3[2] = self.fRec3[1];
			self.fRec3[1] = self.fRec3[0];
			self.fVec1[1] = self.fVec1[0];
			self.fRec1[1] = self.fRec1[0];
		}
	}


}

impl Pedal for TubeScreamer {
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
                    "drive" => {
                        self.fHslider2 = setting.get_value() as f32;
                    }
                    "tone" => {
                        self.fHslider1 = setting.get_value() as f32;
                    }
                    "output" => {
                        self.fHslider0 = setting.get_value() as f32;
                        //
                    }
                    _ => (),
                }
                setting.dirty = false;
            }
        }
    }

    /// This function gets called on a frame of audio.  This is where you filter does what it does.
    fn do_algorithm(&mut self, input: &[f32], output: &mut [f32]) -> () {
        self.compute(input.len() as i32, &[input], &mut[output]);
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
            "name": "Tube Screamer",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_template_pedal {
    use super::*;

    #[test]
    fn can_build() {
        let mut ts = TubeScreamer::new();
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
