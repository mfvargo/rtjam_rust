use std::{sync::mpsc, thread::{self, sleep}, time::Duration};

use log::{error, info};
use rtjam_rust::{
    common::box_error::BoxError, 
    hw_control::{ hw_control_thread::hw_control_thread, status_light::{LightMessage, Color} }
};

fn main() -> Result<(), BoxError> {

    // Turn on the logger
    // std::env::set_var("RUST_LOG", "debug"); // set RUST_LOG environment variable to debug
    env_logger::init();

    info!("starting hardware test");
    let (lights_tx, lights_rx): (mpsc::Sender<LightMessage>, mpsc::Receiver<LightMessage>) =
    mpsc::channel();

    let _hw_handle = thread::spawn(move || {
        let res = hw_control_thread(lights_rx);
        error!("hw control thread exited: {:?}", res);
    });

    let mut pwr = -80.0;
    loop {
        // Toggle the lights
        pwr += 1.0;
        if pwr >= 0.0 {
            pwr = -80.0;
        }
        let _res = lights_tx.send(
            LightMessage{
                input_one: pwr,
                input_two: pwr,
                status: Color::Red,
                blink: true,
        });
        sleep(Duration::new(0, 50_000_000));
    }

    // let _res = hw_handle.join();
    // Ok(())
}
