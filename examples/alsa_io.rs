use std::{sync::mpsc, thread};

use rtjam_rust::{common::box_error::BoxError, pedals::pedal_board::PedalBoard, sound::alsa_thread, JamEngine, ParamMessage};

fn main() -> Result<(), BoxError> {

    // This is the channel the audio engine will use to send us status data
    let (status_data_tx, status_data_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();

    // This is the channel we will use to send commands to the jack engine
    let (command_tx, command_rx): (mpsc::Sender<ParamMessage>, mpsc::Receiver<ParamMessage>) =
        mpsc::channel();

    let (pedal_tx, pedal_rx): (mpsc::Sender<PedalBoard>, mpsc::Receiver<PedalBoard>) =
        mpsc::channel();


    let engine = JamEngine::new(None, status_data_tx, command_rx, pedal_rx, "my_token_here", "gitty_hash", true)?;
    
    // note: add error checking yourself.

    let alsa_handle = thread::spawn(move || {
        let _res = alsa_thread::run(engine);
    });

    let res = alsa_handle.join();
    dbg!(res);
    Ok(())
}
