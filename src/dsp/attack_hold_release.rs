use std::fmt;

use crate::utils::get_coef;
pub struct AttackHoldRelease {
    attack_coef: f64,
    release_coef: f64,
    attack_release_ouput: f64,
    hold_time_count: usize,
    max_hold_time_count: usize,
    last_output: f64,
}

impl AttackHoldRelease {
    pub fn new(attack: f64, hold: f64, release: f64, sample_rate: f64) -> AttackHoldRelease {
        AttackHoldRelease {
            attack_coef: get_coef(attack, sample_rate),
            release_coef: get_coef(release, sample_rate),
            attack_release_ouput: 0.0,
            hold_time_count: 0,
            max_hold_time_count: (hold * sample_rate as f64).round() as usize,
            last_output: 0.0,
        }
    }
    pub fn get(&mut self, trigger: bool) -> f32 {
        if trigger == true {
            self.attack_release_ouput =
                self.attack_coef + (1.0 - self.attack_coef) * self.last_output;
            self.hold_time_count = 0; // reset hold time
        } else {
            self.hold_time_count += 1;
            if self.hold_time_count >= self.max_hold_time_count
            // 20ms hold starts when input goes to 0 - inc and test
            {
                // release if hold time expired
                self.hold_time_count = self.max_hold_time_count; // hold count reset when re-triggered
                self.attack_release_ouput = (1.0 - self.release_coef) * self.last_output;
            }
        }

        self.last_output = self.attack_release_ouput; // trigger(n-1) = trigger(n)

        self.attack_release_ouput as f32
    }
}

impl fmt::Display for AttackHoldRelease {
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
