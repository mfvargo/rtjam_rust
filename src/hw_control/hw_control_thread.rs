//! Thread to manage custom hardware for the pi CM4 hardware varian
//!
//! This thread will initialize the status lights, configure codec hardware
//! and read control knobs from the hardware
use serde_json::Value;
use crate::common::box_error::BoxError;
use std::{sync::mpsc, thread::sleep, time::Duration};

use super::status_light::{StatusFunction, StatusLight, Color};



pub fn hw_control_thread(
    lights_rx: mpsc::Receiver<Value>, // channel from main thread
) -> Result<(), BoxError> {

    let mut input_one = StatusLight::new(StatusFunction::InputOne)?;
    let mut input_two = StatusLight::new(StatusFunction::InputTwo)?;
    let mut status = StatusLight::new(StatusFunction::Status)?;
    // This where we will implement some stuff
    let mut toggle = true;
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
        if toggle {
            input_one.set(Color::Black);
            input_two.set(Color::Black);
            status.set(Color::Black);
        } else {
            input_one.set(Color::Green);
            input_two.set(Color::Red);
            status.set(Color::Orange);
            
        }
        toggle = !toggle;
        sleep(Duration::new(1, 0));
    }
}