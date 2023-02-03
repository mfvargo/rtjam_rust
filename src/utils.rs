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
pub fn clip_float(v: f32) -> f32 {
    if v > 1.0 {
        return 1.0;
    }
    if v < -1.0 {
        return -1.0;
    }
    v
}

pub fn get_frame_power_in_db(frame: &[f32]) -> f32 {
    // linear calcution.  sum of the squares / number of values
    if frame.len() == 0 {
        return to_db(0.0);
    }
    let mut pow: f32 = 0.0;
    for v in frame {
        pow = pow + f32::powi(*v, 2);
    }
    to_db(pow / frame.len() as f32)
}

pub fn to_db(v: f32) -> f32 {
    if v > 0.000_000_1 {
        return 10.0 * f32::log10(v);
    }
    -60.0
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
    fn clip_test() {
        assert_eq!(clip_float(0.1), 0.1);
        assert_eq!(clip_float(-1.3), -1.0);
        assert_eq!(clip_float(2.3), 1.0);
    }
    #[test]
    fn get_frame_power() {
        let frame = [0.0; 128];
        assert_eq!(get_frame_power_in_db(&frame), -60.0);
        let frame = [0.5; 128];
        assert_eq!(get_frame_power_in_db(&frame).round(), -6.0);
    }
}
