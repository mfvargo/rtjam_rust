//! used to track peak envelope using separate attack and release time constants
use num::{Float, FromPrimitive, Zero};
use std::fmt::{self, Display};

use crate::utils::get_coef;

pub struct PeakDetector<T> {
    attack_coef: T,
    release_coef: T,
    peak_detector: T,
    last_output: T,
}

impl<T: Float + FromPrimitive> PeakDetector<T> {
    pub fn build(attack: T, release: T, sample_rate: T) -> PeakDetector<T> {
        PeakDetector {
            attack_coef: get_coef(attack, sample_rate),
            release_coef: get_coef(release, sample_rate),
            peak_detector: Zero::zero(),
            last_output: Zero::zero(),
        }
    }
    pub fn init(&mut self, attack: T, release: T, sample_rate: T) -> () {
        self.attack_coef = get_coef(attack, sample_rate);
        self.release_coef = get_coef(release, sample_rate);
    }

    pub fn get(&mut self, input: T) -> T {
        let inp = input; // .abs();
        if self.peak_detector < inp {
            self.peak_detector = inp * self.attack_coef
                + (T::from_f64(1.0).unwrap() - self.attack_coef) * self.last_output;
        } else {
            self.peak_detector = input * self.release_coef
                + (T::from_f64(1.0).unwrap() - self.release_coef) * self.last_output;
        }
        self.last_output = self.peak_detector;
        self.peak_detector
    }
}

impl<T: Float + FromPrimitive + Display> Display for PeakDetector<T> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ attack: {}, release: {}, peak: {} }}",
            self.attack_coef, self.release_coef, self.peak_detector
        )
    }
}

#[cfg(test)]
mod test_peak_detector {
    use super::*;

    #[test]
    fn get_value() {
        let mut detector: PeakDetector<f32> = PeakDetector::build(0.1, 2.5, 2666.6);
        println!("init: {}", detector);
        // It shoujld start at 0
        assert_eq!(detector.get(0.0), 0.0);
        let samps = vec![0.2, 0.2, 0.4, 0.5, 0.6];
        for v in samps {
            detector.get(v);
        }
        // It should have a peak more than zero
        println!("post: {}", detector);
        assert!(detector.get(0.6) > 0.0);
    }
}
