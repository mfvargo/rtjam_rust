//! grab bag of functions used across the board.  Why are these not in common?
use crate::common::box_error::BoxError;
use num::{Float, FromPrimitive};
use std::env;
use std::fs;

// utility functions

/// function used to get the mac address from the local system.  This address is used to uniquely identify this
/// unit to the rtjam-nation.  TODO:  maybe there is a better way to do this?
pub fn get_my_mac_address(iface: &str) -> Result<String, BoxError> {
    // Get the mac address
    let fcontent = fs::read_to_string(format!("/sys/class/net/{}/address", iface))?;
    Ok(String::from(fcontent.trim()))
}

pub fn get_git_hash() -> String {
    let sha = env!("VERGEN_GIT_SHA");
    String::from(sha)
}

/// Get frame power in dB of a slice of samples
///
/// results are clipped an -60dB which is essentially silence
/// # Example
///
/// ```
///
/// fn main() {
///     let frame = [0.0; 128];
///     assert_eq!(get_frame_power_in_db(&frame), -60.0);
///     let frame = [0.5; 128];
///     assert_eq!(get_frame_power_in_db(&frame).round(), -6.0);
/// }
/// ```
///
pub fn get_frame_power_in_db(frame: &[f32]) -> f64 {
    // linear calcution.  sum of the squares / number of values
    if frame.len() == 0 {
        return to_db(0.0);
    }
    let mut pow: f64 = 0.0;
    for v in frame {
        pow = pow + f64::powi(*v as f64, 2);
    }
    to_db(pow / (frame.len() as f64))
}

// Convert a linear to db
pub fn to_db(v: f64) -> f64 {
    return (10.0 * f64::log10(v)).clamp(-60.0, 100.0);
}

// convert db to linear
pub fn to_lin(v: f64) -> f64 {
    f64::powf(10.0, v / 10.0)
}

/// calculate a filter coefficient give a time constant and sample rate (Darius secret formula)
pub fn get_coef<T: Float + FromPrimitive>(val: T, rate: T) -> T {
    // calculate a filter coef,  Darius secret formula
    let one = T::from_f64(1.0).unwrap();
    let neg_one = T::from_f64(-1.0).unwrap();
    let tau = T::from_f64(2.0 * std::f64::consts::PI).unwrap();
    T::from_i32(27).unwrap() * (one - T::exp(neg_one / (tau * val * rate)))
    // 27.0 * (1.0 - f64::exp(-1.0 * (1.0 / (6.28 * val * rate as f64))))
}

#[cfg(test)]

mod test_utils {
    use super::*;

    #[test]
    fn get_coefficient() {
        let c: f32 = get_coef(0.1, 2666.0);
        println!("Coef: {}", c);
        let c: f64 = get_coef(0.1, 2666.0);
        println!("Coef: {}", c);
    }
    #[test]
    fn get_mac_address() {
        let mac = get_my_mac_address("anbox0").unwrap();
        println!("mac: {}", mac);
    }
    #[test]
    fn get_hash() {
        println!("githash: {}", get_git_hash());
    }

    #[test]
    fn get_frame_power() {
        let frame = [0.0; 128];
        assert_eq!(get_frame_power_in_db(&frame), -60.0);
        let frame = [0.5; 128];
        assert_eq!(get_frame_power_in_db(&frame).round(), -6.0);
    }
    #[test]
    fn lin_to_db_and_back() {
        assert_eq!(to_db(1.0), 0.0);
        assert_eq!(to_lin(-10.0), 0.1);
    }
}
