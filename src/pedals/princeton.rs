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
/// 
/// 
type F32 = f32;
type F64 = f64;

fn mydsp_faustpower2_f(value: F32) -> F32 {
	return value * value;
}
fn mydsp_faustpower3_f(value: F32) -> F32 {
	return value * value * value;
}

pub struct Princeton {
    bypass: bool,
    settings: Vec<PedalSetting<f64>>,
    // Here is where you would add th things you need to make it work
    // BiQuad filter, delays, attack_hold_release, see things in the dsp folder.
    fSampleRate: i32,
	fConst0: F32,
	fConst1: F32,
	fConst2: F32,
	fConst3: F32,
	fConst4: F32,
	fConst5: F32,
	fConst6: F32,
	fConst7: F32,
	fConst8: F32,
	fConst9: F32,
	fConst10: F32,
	fConst11: F32,
	fConst12: F32,
	fConst13: F32,
	fConst14: F32,
	fConst15: F32,
	fConst16: F32,
	fConst17: F32,
	fConst18: F32,
	fConst19: F32,
	fConst20: F32,
	fConst21: F32,
	fConst22: F32,
	fHslider0: F32,
	fConst23: F32,
	fConst24: F32,
	fConst25: F32,
	fConst26: F32,
	fConst27: F32,
	fConst28: F32,
	fConst29: F32,
	fConst30: F32,
	fConst31: F32,
	fConst32: F32,
	fVec0: [F32;2],
	fConst33: F32,
	fConst34: F32,
	fConst35: F32,
	fRec8: [F32;2],
	fVslider0: F32,
	fRec7: [F32;3],
	fRec6: [F32;3],
	fRec5: [F32;4],
	fHslider1: F32,
	fConst36: F32,
	fVslider1: F32,
	fRec4: [F32;3],
	fConst37: F32,
	fRec3: [F32;3],
	fConst38: F32,
	fConst39: F32,
	fConst40: F32,
	fVec1: [F32;2],
	fConst41: F32,
	fConst42: F32,
	fConst43: F32,
	fRec2: [F32;2],
	fRec9: [F32;2],
	fRec1: [F32;3],
	fRec0: [F32;3],
}

impl Princeton {
    /// Construct a new default Princeton
    pub fn new() -> Princeton {
        let mut pedal = Princeton {
            bypass: false,
            settings: Vec::new(),
            fSampleRate: 0,
			fConst0: 0.0,
			fConst1: 0.0,
			fConst2: 0.0,
			fConst3: 0.0,
			fConst4: 0.0,
			fConst5: 0.0,
			fConst6: 0.0,
			fConst7: 0.0,
			fConst8: 0.0,
			fConst9: 0.0,
			fConst10: 0.0,
			fConst11: 0.0,
			fConst12: 0.0,
			fConst13: 0.0,
			fConst14: 0.0,
			fConst15: 0.0,
			fConst16: 0.0,
			fConst17: 0.0,
			fConst18: 0.0,
			fConst19: 0.0,
			fConst20: 0.0,
			fConst21: 0.0,
			fConst22: 0.0,
			fHslider0: 0.0,
			fConst23: 0.0,
			fConst24: 0.0,
			fConst25: 0.0,
			fConst26: 0.0,
			fConst27: 0.0,
			fConst28: 0.0,
			fConst29: 0.0,
			fConst30: 0.0,
			fConst31: 0.0,
			fConst32: 0.0,
			fVec0: [0.0;2],
			fConst33: 0.0,
			fConst34: 0.0,
			fConst35: 0.0,
			fRec8: [0.0;2],
			fVslider0: 0.0,
			fRec7: [0.0;3],
			fRec6: [0.0;3],
			fRec5: [0.0;4],
			fHslider1: 0.0,
			fConst36: 0.0,
			fVslider1: 0.0,
			fRec4: [0.0;3],
			fConst37: 0.0,
			fRec3: [0.0;3],
			fConst38: 0.0,
			fConst39: 0.0,
			fConst40: 0.0,
			fVec1: [0.0;2],
			fConst41: 0.0,
			fConst42: 0.0,
			fConst43: 0.0,
			fRec2: [0.0;2],
			fRec9: [0.0;2],
			fRec1: [0.0;3],
			fRec0: [0.0;3],
        };
        pedal.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "volume",
            vec![],
            9.0,
            0.01,
            10.0,
            0.1,
        ));
        pedal.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "treble",
            vec![],
            7.0,
            0.0,
            10.0,
            0.01,
        ));
        pedal.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::Linear,
            "bass",
            vec![],
            3.5,
            0.0,
            10.0,
            0.01,
        ));
        pedal.settings.push(PedalSetting::new(
            SettingUnit::Continuous,
            SettingType::DB,
            "master",
            vec![],
            1.0,
            0.01,
            1.0,
            0.01,
        ));
        // Initialize the the things you might use.
        pedal.instance_init(48000);
        pedal
    }
    fn instance_reset_params(&mut self) {
		self.fHslider0 = 3.5;
		self.fVslider0 = 9.0;
		self.fHslider1 = 7.0;
		self.fVslider1 = 1.0;
	}
	fn instance_clear(&mut self) {
		for l0 in 0..2 {
			self.fVec0[l0 as usize] = 0.0;
		}
		for l1 in 0..2 {
			self.fRec8[l1 as usize] = 0.0;
		}
		for l2 in 0..3 {
			self.fRec7[l2 as usize] = 0.0;
		}
		for l3 in 0..3 {
			self.fRec6[l3 as usize] = 0.0;
		}
		for l4 in 0..4 {
			self.fRec5[l4 as usize] = 0.0;
		}
		for l5 in 0..3 {
			self.fRec4[l5 as usize] = 0.0;
		}
		for l6 in 0..3 {
			self.fRec3[l6 as usize] = 0.0;
		}
		for l7 in 0..2 {
			self.fVec1[l7 as usize] = 0.0;
		}
		for l8 in 0..2 {
			self.fRec2[l8 as usize] = 0.0;
		}
		for l9 in 0..2 {
			self.fRec9[l9 as usize] = 0.0;
		}
		for l10 in 0..3 {
			self.fRec1[l10 as usize] = 0.0;
		}
		for l11 in 0..3 {
			self.fRec0[l11 as usize] = 0.0;
		}
	}

    fn instance_constants(&mut self, sample_rate: i32) {
		self.fSampleRate = sample_rate;
		self.fConst0 = F32::min(1.92e+05, F32::max(1.0, (self.fSampleRate) as F32));
		self.fConst1 = F32::tan(12566.371 / self.fConst0);
		self.fConst2 = 2.0 * (1.0 - 1.0 / mydsp_faustpower2_f(self.fConst1));
		self.fConst3 = 1.0 / self.fConst1;
		self.fConst4 = (self.fConst3 + -0.76536685) / self.fConst1 + 1.0;
		self.fConst5 = 1.0 / ((self.fConst3 + 0.76536685) / self.fConst1 + 1.0);
		self.fConst6 = (self.fConst3 + -1.847759) / self.fConst1 + 1.0;
		self.fConst7 = 1.0 / ((self.fConst3 + 1.847759) / self.fConst1 + 1.0);
		self.fConst8 = F32::tan(1256.6371 / self.fConst0);
		self.fConst9 = 2.0 * (1.0 - 1.0 / mydsp_faustpower2_f(self.fConst8));
		self.fConst10 = self.fConst0 * F32::sin(2513.2742 / self.fConst0);
		self.fConst11 = 19.82211 / self.fConst10;
		self.fConst12 = 1.0 / self.fConst8;
		self.fConst13 = (self.fConst12 - self.fConst11) / self.fConst8 + 1.0;
		self.fConst14 = (self.fConst12 + self.fConst11) / self.fConst8 + 1.0;
		self.fConst15 = 1.0 / self.fConst14;
		self.fConst16 = F32::tan(282.74335 / self.fConst0);
		self.fConst17 = mydsp_faustpower2_f(self.fConst16);
		self.fConst18 = 2.0 * (1.0 - 1.0 / self.fConst17);
		self.fConst19 = 1.0 / self.fConst16;
		self.fConst20 = (self.fConst19 + -1.4142135) / self.fConst16 + 1.0;
		self.fConst21 = (self.fConst19 + 1.4142135) / self.fConst16 + 1.0;
		self.fConst22 = 1.0 / self.fConst21;
		self.fConst23 = 2.0 * self.fConst0;
		self.fConst24 = mydsp_faustpower2_f(self.fConst23);
		self.fConst25 = mydsp_faustpower3_f(self.fConst23);
		self.fConst26 = F32::tan(7853.9814 / self.fConst0);
		self.fConst27 = 2.0 * (1.0 - 1.0 / mydsp_faustpower2_f(self.fConst26));
		self.fConst28 = 1.0 / self.fConst26;
		self.fConst29 = (self.fConst28 + -0.76536685) / self.fConst26 + 1.0;
		self.fConst30 = 1.0 / ((self.fConst28 + 0.76536685) / self.fConst26 + 1.0);
		self.fConst31 = (self.fConst28 + -1.847759) / self.fConst26 + 1.0;
		self.fConst32 = 1.0 / ((self.fConst28 + 1.847759) / self.fConst26 + 1.0);
		self.fConst33 = 1.0 / F32::tan(219.91148 / self.fConst0);
		self.fConst34 = 1.0 - self.fConst33;
		self.fConst35 = 1.0 / (self.fConst33 + 1.0);
		self.fConst36 = 3.0 * self.fConst25;
		self.fConst37 = 1.0 / (self.fConst17 * self.fConst21);
		self.fConst38 = 3.1415927 / self.fConst10;
		self.fConst39 = (self.fConst12 - self.fConst38) / self.fConst8 + 1.0;
		self.fConst40 = (self.fConst12 + self.fConst38) / self.fConst8 + 1.0;
		self.fConst41 = 1.0 / (self.fConst8 * self.fConst14);
		self.fConst42 = 1.0 - self.fConst12;
		self.fConst43 = 1.0 / (self.fConst12 + 1.0);
	}
	fn instance_init(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate);
		self.instance_reset_params();
		self.instance_clear();
	}

    fn compute(&mut self, count: i32, inputs: &[&[f32]], outputs: &mut[&mut[f32]]) {
        // (&mut self, input: &[f32], output: &mut [f32])
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
		let mut fSlow0: F32 = F32::exp(3.4 * (0.1 * self.fHslider0 + -1.0));
		let mut fSlow1: F32 = self.fConst24 * (0.00012111207 * fSlow0 + 2.7913793e-06);
		let mut fSlow2: F32 = self.fConst25 * (7.44245e-09 * fSlow0 + 1.1431603e-10);
		let mut fSlow3: F32 = 0.0250625 * fSlow0;
		let mut fSlow4: F32 = self.fConst23 * (fSlow3 + 0.01528882);
		let mut fSlow5: F32 = fSlow4 + fSlow2 + (-1.0 - fSlow1);
		let mut fSlow6: F32 = fSlow4 + fSlow1;
		let mut fSlow7: F32 = fSlow6 - 3.0 * (fSlow2 + 1.0);
		let mut fSlow8: F32 = fSlow1 - (fSlow4 + 3.0 * (1.0 - fSlow2));
		let mut fSlow9: F32 = -1.0 - (fSlow6 + fSlow2);
		let mut fSlow10: F32 = 1.0 / fSlow9;
		let mut fSlow11: F32 = 0.5 * mydsp_faustpower2_f(self.fVslider0);
		let mut fSlow12: F32 = self.fHslider1;
		let mut fSlow13: F32 = 9.87e-11 * fSlow0 + fSlow12 * (7.34375e-10 * fSlow0 + 1.128e-11) + 1.516032e-12;
		let mut fSlow14: F32 = self.fConst25 * fSlow13;
		let mut fSlow15: F32 = self.fConst24 * (9.1875e-08 * fSlow12 + 3.61207e-06 * fSlow0 + 6.78294e-08);
		let mut fSlow16: F32 = self.fConst23 * (fSlow3 + 6.25e-06 * fSlow12 + 0.00052632);
		let mut fSlow17: F32 = fSlow16 + fSlow15;
		let mut fSlow18: F32 = fSlow17 + fSlow14;
		let mut fSlow19: F32 = fSlow16 + fSlow14 - fSlow15;
		let mut fSlow20: F32 = self.fConst36 * fSlow13;
		let mut fSlow21: F32 = fSlow17 - fSlow20;
		let mut fSlow22: F32 = fSlow15 + fSlow20 - fSlow16;
		let mut fSlow23: F32 = mydsp_faustpower2_f(self.fVslider1) / fSlow9;
		let zipped_iterators = inputs0.zip(outputs0);
		for (input0, output0) in zipped_iterators {
			let mut fTemp0: F32 = self.fConst9 * self.fRec3[1];
			let mut fTemp1: F32 = *input0;
			self.fVec0[0] = fTemp1;
			self.fRec8[0] = -(self.fConst35 * (self.fConst34 * self.fRec8[1] - self.fConst33 * (fTemp1 - self.fVec0[1])));
			self.fRec7[0] = fSlow11 * (self.fRec8[0] / (F32::abs(fSlow11 * self.fRec8[0]) + 1.0)) - self.fConst32 * (self.fConst31 * self.fRec7[2] + self.fConst27 * self.fRec7[1]);
			self.fRec6[0] = self.fConst32 * (self.fRec7[2] + self.fRec7[0] + 2.0 * self.fRec7[1]) - self.fConst30 * (self.fConst29 * self.fRec6[2] + self.fConst27 * self.fRec6[1]);
			self.fRec5[0] = self.fConst30 * (self.fRec6[2] + self.fRec6[0] + 2.0 * self.fRec6[1]) - fSlow10 * (fSlow8 * self.fRec5[1] + fSlow7 * self.fRec5[2] + fSlow5 * self.fRec5[3]);
			let mut fTemp2: F32 = fSlow22 * self.fRec5[1] + fSlow21 * self.fRec5[2] + fSlow19 * self.fRec5[3] - fSlow18 * self.fRec5[0];
			self.fRec4[0] = fSlow23 * (fTemp2 / (F32::abs(fSlow23 * fTemp2) + 1.0)) - self.fConst22 * (self.fConst20 * self.fRec4[2] + self.fConst18 * self.fRec4[1]);
			self.fRec3[0] = self.fConst37 * (self.fRec4[2] + (self.fRec4[0] - 2.0 * self.fRec4[1])) - self.fConst15 * (self.fConst13 * self.fRec3[2] + fTemp0);
			let mut fTemp3: F32 = fTemp0 + self.fConst40 * self.fRec3[0] + self.fConst39 * self.fRec3[2];
			self.fVec1[0] = fTemp3;
			self.fRec2[0] = -(self.fConst43 * (self.fConst42 * self.fRec2[1] - self.fConst41 * (fTemp3 - self.fVec1[1])));
			self.fRec9[0] = -(self.fConst43 * (self.fConst42 * self.fRec9[1] - self.fConst15 * (fTemp3 + self.fVec1[1])));
			self.fRec1[0] = self.fRec9[0] + 1.7782794 * self.fRec2[0] - self.fConst7 * (self.fConst6 * self.fRec1[2] + self.fConst2 * self.fRec1[1]);
			self.fRec0[0] = self.fConst7 * (self.fRec1[2] + self.fRec1[0] + 2.0 * self.fRec1[1]) - self.fConst5 * (self.fConst4 * self.fRec0[2] + self.fConst2 * self.fRec0[1]);
			*output0 = self.fConst5 * (self.fRec0[2] + self.fRec0[0] + 2.0 * self.fRec0[1]);
			self.fVec0[1] = self.fVec0[0];
			self.fRec8[1] = self.fRec8[0];
			self.fRec7[2] = self.fRec7[1];
			self.fRec7[1] = self.fRec7[0];
			self.fRec6[2] = self.fRec6[1];
			self.fRec6[1] = self.fRec6[0];
			for j0 in (1..=3).rev() {
				self.fRec5[j0 as usize] = self.fRec5[(i32::wrapping_sub(j0, 1)) as usize];
			}
			self.fRec4[2] = self.fRec4[1];
			self.fRec4[1] = self.fRec4[0];
			self.fRec3[2] = self.fRec3[1];
			self.fRec3[1] = self.fRec3[0];
			self.fVec1[1] = self.fVec1[0];
			self.fRec2[1] = self.fRec2[0];
			self.fRec9[1] = self.fRec9[0];
			self.fRec1[2] = self.fRec1[1];
			self.fRec1[1] = self.fRec1[0];
			self.fRec0[2] = self.fRec0[1];
			self.fRec0[1] = self.fRec0[0];
		}
	} 
	
}

impl Pedal for Princeton {
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
                    "volume" => {
                        self.fVslider0 = setting.get_value() as f32;
                    }
                    "master" => {
                        self.fVslider1 = setting.get_value() as f32;
                    }
                    "treble" => {
                        self.fHslider1 = setting.get_value() as f32;
                        //
                    }
                    "bass" => {
                        self.fHslider0 = setting.get_value() as f32;
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
            "name": "Princeton",
            "settings": settings,
        })
    }
}

#[cfg(test)]

mod test_template_pedal {
    use super::*;

    #[test]
    fn can_build() {
        let mut ts = Princeton::new();
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
