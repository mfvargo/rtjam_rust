//! averaging filter use to smooth sequences
use std::fmt;

use crate::utils::get_coef;

pub struct SmoothingFilter {
    coef: f64,
    last_output: f64,
}

impl SmoothingFilter {
    pub fn build(time_const: f64, sample_rate: f64) -> SmoothingFilter {
        SmoothingFilter {
            coef: get_coef(time_const, sample_rate),
            last_output: 0.0,
        }
    }

    pub fn get(&mut self, input: f64) -> f64 {
        self.last_output = input * self.coef + (1.0 - self.coef) * self.last_output;
        self.last_output
    }
    pub fn get_last_output(&self) -> f64 {
        self.last_output
    }
}

impl fmt::Display for SmoothingFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ coef: {}, last_output: {} }}",
            self.coef, self.last_output
        )
    }
}

#[cfg(test)]
mod test_smoothing_filter {
    use super::*;

    #[test]
    fn get_value() {
        let mut filter = SmoothingFilter::build(2.5, 2666.6);
        println!("init: {}", filter);
        // It shoujld start at 0
        assert_eq!(filter.get(0.0), 0.0);
        let samps = vec![0.2, 0.2, 0.4, 0.5, 0.6];
        for v in samps {
            filter.get(v);
        }
        // It should have a peak more than zero
        println!("post: {}", filter);
        assert!(filter.get(0.6) > 0.0);
    }
}
