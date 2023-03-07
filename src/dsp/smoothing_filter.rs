use num::{Float, FromPrimitive, Zero};
use std::fmt::{self, Display};

use crate::utils::get_coef;

pub struct SmoothingFilter<T> {
    coef: T,
    last_output: T,
}

impl<T: Float + FromPrimitive> SmoothingFilter<T> {
    pub fn build(time_const: T, sample_rate: T) -> SmoothingFilter<T> {
        SmoothingFilter {
            coef: get_coef(time_const, sample_rate),
            last_output: Zero::zero(),
        }
    }

    pub fn get(&mut self, input: T) -> T {
        let one = T::from_i32(1).unwrap();
        self.last_output = input * self.coef + (one - self.coef) * self.last_output;
        self.last_output
    }
    pub fn get_last_output(&self) -> T {
        self.last_output
    }
}

impl<T: Float + FromPrimitive + Display> Display for SmoothingFilter<T> {
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
