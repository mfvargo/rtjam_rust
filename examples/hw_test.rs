use std::{sync::mpsc, thread};

use log::{error, info};
use rtjam_rust::{common::box_error::BoxError, hw_control::{hw_control_thread::hw_control_thread, status_light::LightMessage}};

fn main() -> Result<(), BoxError> {

    // Turn on the logger
    env_logger::init();

    info!("starting hardware test");
    let (_lights_tx, lights_rx): (mpsc::Sender<LightMessage>, mpsc::Receiver<LightMessage>) =
    mpsc::channel();

    let hw_handle = thread::spawn(move || {
        let res = hw_control_thread(lights_rx);
        error!("hw control thread exited: {:?}", res);
    });

    let _res = hw_handle.join();
    Ok(())
}
