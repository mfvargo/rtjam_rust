use std::fmt;
pub struct PeakDetector {
    attack_coef: f64,
    release_coef: f64,
    peak_detector: f64,
    last_output: f64,
}

impl PeakDetector {
    pub fn build(attack: f64, release: f64, sample_rate: u32) -> PeakDetector {
        PeakDetector {
            attack_coef: Self::get_coef(attack, sample_rate),
            release_coef: Self::get_coef(release, sample_rate),
            peak_detector: 0.0,
            last_output: 0.0,
        }
    }
    fn get_coef(val: f64, rate: u32) -> f64 {
        // calculate a filter coef,  Darius secret formula
        27.0 * (1.0 - f64::exp(-1.0 * (1.0 / (6.28 * val * rate as f64))))
    }

    pub fn get(&mut self, input: f64) -> f64 {
        let inp = input; // .abs();
        if self.peak_detector < inp {
            self.peak_detector =
                inp * self.attack_coef + (1.0 - self.attack_coef) * self.last_output;
        } else {
            self.peak_detector =
                input * self.release_coef + (1.0 - self.release_coef) * self.last_output;
        }
        self.last_output = self.peak_detector;
        self.peak_detector
    }
}

impl fmt::Display for PeakDetector {
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
        let mut detector = PeakDetector::build(0.1, 2.5, 2666);
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
