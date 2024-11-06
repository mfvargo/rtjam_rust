use std::sync::mpsc;

use rtjam_rust::{common::box_error::BoxError, pedals::pedal_board::PedalBoard, sound::alsa_thread, JamEngine, ParamMessage};
use thread_priority::*;

fn main() -> Result<(), BoxError> {

    // This is the channel the audio engine will use to send us status data
    let (status_data_tx, _status_data_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();

    // This is the channel we will use to send commands to the jack engine
    let (_command_tx, command_rx): (mpsc::Sender<ParamMessage>, mpsc::Receiver<ParamMessage>) =
        mpsc::channel();

    let (_pedal_tx, pedal_rx): (mpsc::Sender<PedalBoard>, mpsc::Receiver<PedalBoard>) =
        mpsc::channel();


    let engine = JamEngine::new(None, status_data_tx, command_rx, pedal_rx, "my_token_here", "gitty_hash", true)?;
    
    // note: add error checking yourself.

    let builder = ThreadBuilder::default()
                        .name("Real-Time Thread".to_string())
                        .priority(ThreadPriority::Max);
                        
                        // .spawn_with_priority(ThreadPriority::Max, f);

                    //     .priority(std::thread::Priority::Realtime); 

    let alsa_handle = builder.spawn(move |_result| {
        let _res = alsa_thread::run(engine);
    })?;

    let _e = alsa_handle.join();
    Ok(())
}
