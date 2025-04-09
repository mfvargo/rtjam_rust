//! Thread to manage custom hardware for the pi CM4 hardware varian
//!
//! This thread will initialize the status lights, configure codec hardware
//! and read control knobs from the hardware
use log::{debug, error, info};

use crate::common::box_error::BoxError;
use std::{sync::mpsc, thread::sleep, time::Duration};

use super::{codec_control::CodecControl, status_light::{HardwareMessage, StatusFunction, StatusLight}};



pub fn hw_control_thread(
    lights_rx: mpsc::Receiver<HardwareMessage>, // channel from main thread
) -> Result<(), BoxError> {

    let mut inp_one = StatusLight::new(StatusFunction::InputOne)?;
    let mut inp_two = StatusLight::new(StatusFunction::InputTwo)?;
    let mut stat_light = StatusLight::new(StatusFunction::Status)?;
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
                match m {
                    HardwareMessage::LightMessage { input_one, input_two, status, blink } => {
                        // Got a light update
                        debug!("input_one: {}, input_two: {}, status: {:?}, blink: {}", input_one, input_two, status, blink);
                        inp_one.power(input_one);
                        inp_two.power(input_two);
                        stat_light.set(status, blink);
                    }
                    HardwareMessage::GainMessage { input_1_gain, input_2_gain, headphone_gain } => {
                        info!("input_1_gain: {}, input_2_gain: {}, headphone_gain: {}", input_1_gain, input_2_gain, headphone_gain);
                        match codec_option {
                            Some(ref mut codec) => {
                                codec.update_input_one_gain(input_1_gain)?;
                                codec.update_input_two_gain(input_2_gain)?;
                                codec.update_headphone_gain(headphone_gain)?;
                            }
                            None => {
                                error!("No codec available to set gain");
                            }
                        }
                    }
                    // _ => {
                    //     // do nothing
                    // }
                }
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