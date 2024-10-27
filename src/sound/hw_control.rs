//! Thread to manage custom hardware for the pi CM4 hardware varian
//!
//! This thread will initialize the status lights, configure codec hardware
//! and read control knobs from the hardware
use serde_json::Value;
use crate::common::box_error::BoxError;
use std::{sync::mpsc, thread::sleep, time::Duration};

pub fn hw_control_thread(
    lights_rx: mpsc::Receiver<Value>, // channel from main thread
) -> Result<(), BoxError> {
    // This where we will implement some stuff
    loop {
        let res = lights_rx.try_recv();
        match res {
            Ok(m) => {
                // Got a message to send
                println!("received light status message: {}", m);
            }
            Err(_e) => {
                // dbg!(_e);
            }
        }
        sleep(Duration::new(0, 1000000));
    }
}