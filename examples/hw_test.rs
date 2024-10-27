use std::{sync::mpsc, thread};
use serde_json::Value;

use rtjam_rust::{common::box_error::BoxError, hw_control::hw_control_thread::hw_control_thread};

fn main() -> Result<(), BoxError> {
    // note: add error checking yourself.

    let (_lights_tx, lights_rx): (mpsc::Sender<Value>, mpsc::Receiver<Value>) =
    mpsc::channel();

    let hw_handle = thread::spawn(move || {
        let _res = hw_control_thread(lights_rx);
    });

    let _res = hw_handle.join();
    Ok(())
}
