
//! A PitchDetector from Faust an.pitchdetector()
//! 

// These macros are to make the Faust generated code compile quietly
#![allow(unused_parens)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(non_upper_case_globals)]


type F32 = f32;
type F64 = f64;


pub type FaustFloat = F32;
use std::convert::TryInto;
fn mydsp_faustpower2_f(value: F32) -> F32 {
	return value * value;
}
// mod ffi {
// 	use std::os::raw::c_float;
// 	// Conditionally compile the link attribute only on non-Windows platforms
// 	#[cfg_attr(not(target_os = "windows"), link(name = "m"))]
// 	extern "C" {
// 		pub fn remainderf(from: c_float, to: c_float) -> c_float;
// 		pub fn rintf(val: c_float) -> c_float;
// 	}
// }
// fn remainder_f32(from: f32, to: f32) -> f32 {
// 	unsafe { ffi::remainderf(from, to) }
// }
// fn rint_f32(val: f32) -> f32 {
// 	unsafe { ffi::rintf(val) }
// }

pub struct PitchDetector {
	fHslider0: F32,
	fSampleRate: i32,
	fConst0: F32,
	fConst1: F32,
	fConst2: F32,
	fVec0: [F32;2],
	fConst3: F32,
	fConst4: F32,
	fConst5: F32,
	fRec4: [F32;2],
	fRec3: [F32;3],
	fRec2: [F32;3],
	fVec1: [F32;2],
	fRec1: [F32;2],
	fConst6: F32,
	fRec0: [F32;2],
    note: F32,
}

impl PitchDetector {

	pub fn new() -> PitchDetector { 
		let mut det = PitchDetector {
			fHslider0: 0.1,
			fSampleRate: 48_000,
			fConst0: 0.0,
			fConst1: 0.0,
			fConst2: 0.0,
			fVec0: [0.0;2],
			fConst3: 0.0,
			fConst4: 0.0,
			fConst5: 0.0,
			fRec4: [0.0;2],
			fRec3: [0.0;3],
			fRec2: [0.0;3],
			fVec1: [0.0;2],
			fRec1: [0.0;2],
			fConst6: 0.0,
			fRec0: [0.0;2],
            note: 0.0,
		};
        det.init(48000);
        det
	}
	
	pub fn instance_reset_params(&mut self) {
		self.fHslider0 = 0.45;
	}
	pub fn instance_clear(&mut self) {
		for l0 in 0..2 {
			self.fVec0[l0 as usize] = 0.0;
		}
		for l1 in 0..2 {
			self.fRec4[l1 as usize] = 0.0;
		}
		for l2 in 0..3 {
			self.fRec3[l2 as usize] = 0.0;
		}
		for l3 in 0..3 {
			self.fRec2[l3 as usize] = 0.0;
		}
		for l4 in 0..2 {
			self.fVec1[l4 as usize] = 0.0;
		}
		for l5 in 0..2 {
			self.fRec1[l5 as usize] = 0.0;
		}
		for l6 in 0..2 {
			self.fRec0[l6 as usize] = 0.0;
		}
	}
	pub fn instance_constants(&mut self, sample_rate: i32) {
		self.fSampleRate = sample_rate;
		self.fConst0 = F32::min(1.92e+05, F32::max(1.0, (self.fSampleRate) as F32));
		self.fConst1 = 1.0 / self.fConst0;
		self.fConst2 = 3.1415927 / self.fConst0;
		self.fConst3 = 1.0 / F32::tan(62.831852 / self.fConst0);
		self.fConst4 = 1.0 - self.fConst3;
		self.fConst5 = 1.0 / (self.fConst3 + 1.0);
		self.fConst6 = 0.5 * self.fConst0;
	}
	pub fn instance_init(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate);
		self.instance_reset_params();
		self.instance_clear();
	}
	pub fn init(&mut self, sample_rate: i32) {
		self.instance_init(sample_rate);
	}
	
    pub fn do_compute(&mut self, input: &[F32], output: &mut[F32]) -> F32 {
        self.note = 0.0;
		let inputs0 = input.iter();
		let outputs0 = output.iter_mut();
		let mut fSlow0: F32 = 1.0 * self.fHslider0;
		let mut iSlow1: i32 = (F32::abs(fSlow0) < 1.1920929e-07) as i32;
		let mut fSlow2: F32 = (if iSlow1 != 0 {0.0} else {F32::exp(-(self.fConst1 / (if iSlow1 != 0 {1.0} else {fSlow0})))});
		let mut fSlow3: F32 = 1.0 - fSlow2;
		let zipped_iterators = inputs0.zip(outputs0);
		for (input0, output0) in zipped_iterators {
			let mut fTemp0: F32 = F32::tan(self.fConst2 * F32::max(2e+01, self.fRec0[1]));
			let mut fTemp1: F32 = 1.0 / fTemp0;
			let mut fTemp2: F32 = (fTemp1 + 0.76536685) / fTemp0 + 1.0;
			let mut fTemp3: F32 = 1.0 - 1.0 / mydsp_faustpower2_f(fTemp0);
			let mut fTemp4: F32 = (fTemp1 + 1.847759) / fTemp0 + 1.0;
			let mut fTemp5: F32 = *input0;
			self.fVec0[0] = fTemp5;
			self.fRec4[0] = -(self.fConst5 * (self.fConst4 * self.fRec4[1] - self.fConst3 * (fTemp5 - self.fVec0[1])));
			self.fRec3[0] = self.fRec4[0] - (self.fRec3[2] * ((fTemp1 + -1.847759) / fTemp0 + 1.0) + 2.0 * self.fRec3[1] * fTemp3) / fTemp4;
			self.fRec2[0] = (self.fRec3[2] + self.fRec3[0] + 2.0 * self.fRec3[1]) / fTemp4 - (self.fRec2[2] * ((fTemp1 + -0.76536685) / fTemp0 + 1.0) + 2.0 * fTemp3 * self.fRec2[1]) / fTemp2;
			let mut fTemp6: F32 = self.fRec2[2] + self.fRec2[0] + 2.0 * self.fRec2[1];
			self.fVec1[0] = fTemp6 / fTemp2;
			self.fRec1[0] = fSlow3 * (((fTemp6 * self.fVec1[1] / fTemp2) < 0.0) as i32) as u32 as F32 + fSlow2 * self.fRec1[1];
			self.fRec0[0] = self.fConst6 * self.fRec1[0];
            self.note += self.fRec0[0];
			*output0 = self.fRec0[0];
			self.fVec0[1] = self.fVec0[0];
			self.fRec4[1] = self.fRec4[0];
			self.fRec3[2] = self.fRec3[1];
			self.fRec3[1] = self.fRec3[0];
			self.fRec2[2] = self.fRec2[1];
			self.fRec2[1] = self.fRec2[0];
			self.fVec1[1] = self.fVec1[0];
			self.fRec1[1] = self.fRec1[0];
			self.fRec0[1] = self.fRec0[0];
		}
        self.note = self.note / output.len() as F32;
        self.note
    }

	pub fn compute_arrays(&mut self, count: usize, inputs: &[&[FaustFloat] ; 1], outputs: &mut [&mut [FaustFloat] ; 1]) {
		
		let [inputs0, ] = inputs;
		let inputs0 = inputs0[..count as usize].iter();
		let [outputs0, ] = outputs;
		let outputs0 = outputs0[..count as usize].iter_mut();
		let mut fSlow0: F32 = 1.0 * self.fHslider0;
		let mut iSlow1: i32 = (F32::abs(fSlow0) < 1.1920929e-07) as i32;
		let mut fSlow2: F32 = (if iSlow1 != 0 {0.0} else {F32::exp(-(self.fConst1 / (if iSlow1 != 0 {1.0} else {fSlow0})))});
		let mut fSlow3: F32 = 1.0 - fSlow2;
		let zipped_iterators = inputs0.zip(outputs0);
		for (input0, output0) in zipped_iterators {
			let mut fTemp0: F32 = F32::tan(self.fConst2 * F32::max(2e+01, self.fRec0[1]));
			let mut fTemp1: F32 = 1.0 / fTemp0;
			let mut fTemp2: F32 = (fTemp1 + 0.76536685) / fTemp0 + 1.0;
			let mut fTemp3: F32 = 1.0 - 1.0 / mydsp_faustpower2_f(fTemp0);
			let mut fTemp4: F32 = (fTemp1 + 1.847759) / fTemp0 + 1.0;
			let mut fTemp5: F32 = *input0;
			self.fVec0[0] = fTemp5;
			self.fRec4[0] = -(self.fConst5 * (self.fConst4 * self.fRec4[1] - self.fConst3 * (fTemp5 - self.fVec0[1])));
			self.fRec3[0] = self.fRec4[0] - (self.fRec3[2] * ((fTemp1 + -1.847759) / fTemp0 + 1.0) + 2.0 * self.fRec3[1] * fTemp3) / fTemp4;
			self.fRec2[0] = (self.fRec3[2] + self.fRec3[0] + 2.0 * self.fRec3[1]) / fTemp4 - (self.fRec2[2] * ((fTemp1 + -0.76536685) / fTemp0 + 1.0) + 2.0 * fTemp3 * self.fRec2[1]) / fTemp2;
			let mut fTemp6: F32 = self.fRec2[2] + self.fRec2[0] + 2.0 * self.fRec2[1];
			self.fVec1[0] = fTemp6 / fTemp2;
			self.fRec1[0] = fSlow3 * (((fTemp6 * self.fVec1[1] / fTemp2) < 0.0) as i32) as u32 as F32 + fSlow2 * self.fRec1[1];
			self.fRec0[0] = self.fConst6 * self.fRec1[0];
			*output0 = self.fRec0[0];
			self.fVec0[1] = self.fVec0[0];
			self.fRec4[1] = self.fRec4[0];
			self.fRec3[2] = self.fRec3[1];
			self.fRec3[1] = self.fRec3[0];
			self.fRec2[2] = self.fRec2[1];
			self.fRec2[1] = self.fRec2[0];
			self.fVec1[1] = self.fVec1[0];
			self.fRec1[1] = self.fRec1[0];
			self.fRec0[1] = self.fRec0[0];
		}
	}

	
	pub fn compute(&mut self, count: usize, inputs: & [& [FaustFloat] ], outputs: & mut[& mut[FaustFloat] ]) {
		let input_array = inputs.split_at(1).0.try_into().expect("too few input buffers");
		let output_array = outputs.split_at_mut(1).0.try_into().expect("too few output buffers");
		self.compute_arrays(count, input_array, output_array);
	}

}