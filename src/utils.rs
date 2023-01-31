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
}
