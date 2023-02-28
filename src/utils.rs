use crate::common::box_error::BoxError;
use std::env;
use std::fs;

// utility functions

pub fn get_my_mac_address(iface: &str) -> Result<String, BoxError> {
    // Get the mac address
    let fcontent = fs::read_to_string(format!("/sys/class/net/{}/address", iface))?;
    Ok(String::from(fcontent.trim()))
}

pub fn get_git_hash() -> String {
    let sha = env!("VERGEN_GIT_SHA");
    String::from(sha)
}

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

#[cfg(test)]

mod test_utils {
    use super::*;

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
