//! Base asbtraction for an envelop filter.
//!
//!```text
//!  Envelope Filter Effect - Block Diagram
//!
//!
//!                              +----------+
//!            ----------------->| Dry Gain |------------------------
//!            |                 |          |                       |
//!            |                 +----------+                       |
//!            |                                                    |
//!            |                                                    |
//!            |               +---------------+                    |
//!            |               | Biquad Filter |                    |
//!            |               |      (2)      |                    v
//!  Input ------------------->| - 4 pole      |     +-----+    +------+
//!               |            | - LPF/BPF/HPF |---->|level|--->| sum  |---> Output
//!               |            |               |     +-----+    +------+
//!               |            |               |
//!               |            +---------------+
//!               |                     ^
//!               |                     |
//!               v              +--------------+
//!        +----------------+    |Coeff Control |
//!        | Peak Detector  |    |              |
//!        |                |--->| Sens/Freq/Q  |
//!        | -attack/release|    |              |
//!        +----------------+    | - Fs/4       |
//!                              +--------------+
//!```
use std::fmt;

use crate::dsp::{
    biquad::{BiQuadFilter, FilterType},
    peak_detector::PeakDetector,
};

pub struct EnvelopeBase {
    filter: BiQuadFilter,
    peak: PeakDetector<f64>,
    update_idx: usize,
    pub ftype: FilterType,
    pub freq: f64,
    pub resonance: f64,
    pub sensitivity: f64,
    pub attack: f64,
    pub release: f64,
    pub level: f64,
    pub dry: f64,
}

const UPDATE_RATE: usize = 4;

impl EnvelopeBase {
    pub fn new() -> EnvelopeBase {
        let mut env = EnvelopeBase {
            ftype: FilterType::LowPass,
            filter: BiQuadFilter::new(),
            peak: PeakDetector::build(0.01, 0.1, 48_000.0),
            update_idx: 0,
            freq: 250.0,
            resonance: 4.0,
            sensitivity: 1.0,
            attack: 0.01,
            release: 0.1,
            level: 1.0,
            dry: 0.1,
        };
        env.init();
        env
    }

    pub fn init(&mut self) {
        self.filter
            .init(self.ftype, self.freq, 1.0, self.resonance, 48_000.0);
        self.peak.init(self.attack, self.release, 48_000.0);
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) -> () {
        // Implement the delay
        let mut i = 0;
        for sample in input {
            let samp = *sample as f64;
            let mut value = self.peak.get(samp); // calculate magnitude of incoming signal
            value = (value * self.sensitivity * 200.0) + self.freq; // apply gain to envelope, add in freq knob (start freq)

            self.update_idx += 1;
            if self.update_idx % UPDATE_RATE == 0 {
                self.update_idx = 0;
                self.filter
                    .init(self.ftype, value, 1.0, self.resonance, 48_000.0);
            }

            // apply 4th order dynamic filter
            value = 0.5 * self.filter.get_sample_64(&samp);
            value = self.filter.get_sample_64(&value);

            match self.ftype {
                FilterType::BandPass => {
                    value *= 12.0;
                }
                FilterType::HighPass => {
                    value *= 2.0;
                }
                _ => (),
            }

            value = value * self.level  // output level
                          + self.dry * samp; // output + dry

            output[i] = value as f32;
            i += 1;
        }
    }
}

impl fmt::Display for EnvelopeBase {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ sample: {}, freq: {}, res: {}, attack: {}, release: {} }}",
            self.update_idx, self.freq, self.resonance, self.attack, self.release,
        )
    }
}
#[cfg(test)]
mod test_delay_base {

    use super::*;

    #[test]
    fn can_build_and_run() {
        let mut env = EnvelopeBase::new();
        env.init();
        let input = [1.0; 128];
        let mut output = [1.0; 128];
        env.process(&input, &mut output);
        println!("out: {:?}", output);
    }
}
