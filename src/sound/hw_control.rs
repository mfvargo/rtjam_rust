//! Thread to manage custom hardware for the pi CM4 hardware varian
//!
//! This thread will initialize the status lights, configure codec hardware
//! and read control knobs from the hardware
use serde_json::Value;
use crate::common::box_error::BoxError;
use std::{sync::mpsc, thread::sleep, time::Duration};

use rppal::gpio::Gpio;

const GPIO_LED: u8 = 23;


pub fn hw_control_thread(
    lights_rx: mpsc::Receiver<Value>, // channel from main thread
) -> Result<(), BoxError> {
    // This where we will implement some stuff

    match Gpio::new() {
        Ok(_gpio) => {
            // Looks good so just continue on...
        }
        Err(e) => {
            // No gpio, exit this thread
            dbg!(e);
            return Ok(());
        }
    }

    let mut pin = Gpio::new()?.get(GPIO_LED)?.into_output();
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
            pin.set_high();
        } else {
            pin.set_low();
        }
        toggle = !toggle;
        sleep(Duration::new(1, 0));
    }
}