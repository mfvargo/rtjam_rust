use std::fmt::{self, Display};

use num::{Float, FromPrimitive, Zero};

use crate::utils::get_coef;
pub struct AttackHoldRelease<T> {
    attack_coef: T,
    release_coef: T,
    attack_release_ouput: T,
    hold_time_count: usize,
    max_hold_time_count: usize,
    last_output: T,
}

impl<T: Float + FromPrimitive> AttackHoldRelease<T> {
    pub fn one() -> T {
        T::from_i64(1).unwrap()
    }
    pub fn new(attack: T, hold: T, release: T, sample_rate: T) -> AttackHoldRelease<T> {
        AttackHoldRelease {
            attack_coef: get_coef(attack, sample_rate),
            release_coef: get_coef(release, sample_rate),
            attack_release_ouput: Zero::zero(),
            hold_time_count: 0,
            max_hold_time_count: (hold * sample_rate).round().to_usize().unwrap(),
            last_output: Zero::zero(),
        }
    }
    pub fn get(&mut self, trigger: bool) -> T {
        if trigger == true {
            self.attack_release_ouput =
                self.attack_coef + (Self::one() - self.attack_coef) * self.last_output;
            self.hold_time_count = 0; // reset hold time
        } else {
            self.hold_time_count += 1;
            if self.hold_time_count >= self.max_hold_time_count
            // 20ms hold starts when input goes to 0 - inc and test
            {
                // release if hold time expired
                self.hold_time_count = self.max_hold_time_count; // hold count reset when re-triggered
                self.attack_release_ouput = (Self::one() - self.release_coef) * self.last_output;
            }
        }

        self.last_output = self.attack_release_ouput; // trigger(n-1) = trigger(n)

        self.attack_release_ouput
    }
}

impl<T: Float + FromPrimitive + Display> Display for AttackHoldRelease<T> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ attack: {}, release: {}, hold_count: {} }}",
            self.attack_coef, self.release_coef, self.hold_time_count
        )
    }
}

#[cfg(test)]
mod test_peak_detector {
    use super::*;

    #[test]
    fn get_value() {
        let mut detector = AttackHoldRelease::new(0.1, 0.5, 2.5, 2666.6);
        println!("init: {}", detector);
        // It should start at 0
        assert!(detector.get(true) > 0.0);
    }
}
