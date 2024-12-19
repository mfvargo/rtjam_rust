//! Thread to manage custom hardware for the pi CM4 hardware varian
//!
//! This thread will initialize the status lights, configure codec hardware
//! and read control knobs from the hardware
use log::{debug, error};

use crate::common::box_error::BoxError;
use std::{sync::mpsc, thread::sleep, time::Duration};

use super::{codec_control::CodecControl, status_light::{LightMessage, StatusFunction, StatusLight}};



pub fn hw_control_thread(
    lights_rx: mpsc::Receiver<LightMessage>, // channel from main thread
) -> Result<(), BoxError> {

    let mut input_one = StatusLight::new(StatusFunction::InputOne)?;
    let mut input_two = StatusLight::new(StatusFunction::InputTwo)?;
    let mut status = StatusLight::new(StatusFunction::Status)?;
    let mut codec_option: Option<CodecControl> = None;

    match CodecControl::new() {
        Ok(codec) => {
            codec_option = Some(codec);
            println!("codec initiated");
        }
        Err(e) => {
            error!("{}", e);
        }
    }

    // This where we will implement some stuff
    loop {
        // poll the message queue
        let res = lights_rx.try_recv();
        match res {
            Ok(m) => {
                // Got a light update
                debug!("setting lights: {:?}", &m);
                input_one.power(m.input_one);
                input_two.power(m.input_two);
                status.set(m.status, m.blink);
            }
            Err(_e) => {
                // nothing to read right now
                // dbg!(_e);
            }
        }
        // Now check the pots
        match codec_option {
            Some(ref mut codec) => {
                codec.read_pots();
        //        debug!("codec: {}", &codec);
            }
            None => {
                // No codec could be constructed.  Just ignore it
            }
        }
        sleep(Duration::new(0, 5_000_000));
    }
}