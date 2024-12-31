use std::{sync::mpsc, thread::sleep, time::Duration};

use rtjam_rust::{common::box_error::BoxError, sound::alsa_thread, JamEngine, ParamMessage};
use pedal_board::pedals::pedal_board::PedalBoard;
use thread_priority::*;
use log::{trace, error};


fn main() -> Result<(), BoxError> {

    // Turn on the logger
    env_logger::init();


    // This is the channel the audio engine will use to send us status data
    let (status_data_tx, status_data_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();

    // This is the channel we will use to send commands to the jack engine
    let (_command_tx, command_rx): (mpsc::Sender<ParamMessage>, mpsc::Receiver<ParamMessage>) =
        mpsc::channel();

    let (_pedal_tx, pedal_rx): (mpsc::Sender<PedalBoard>, mpsc::Receiver<PedalBoard>) =
        mpsc::channel();


    let mut engine = JamEngine::new(None, status_data_tx, command_rx, pedal_rx, "my_token_here", "gitty_hash", false)?;
    
    // note: add error checking yourself.

    let builder = ThreadBuilder::default()
                        .name("Real-Time Thread".to_string())
                        .priority(ThreadPriority::Max);
                        
                        // .spawn_with_priority(ThreadPriority::Max, f);

                    //     .priority(std::thread::Priority::Realtime); 

    let alsa_handle = builder.spawn(move |_result| {
        match alsa_thread::run(&mut engine, "hw:CODEC", "hw:CODEC") {
            Ok(()) => {
                error!("alsa ended with OK");
            }
            Err(e) => {
                error!("alsa exited with error {}", e);
            }
            
        }
    })?;

    // Read data from JamEngine
    while !alsa_handle.is_finished() {
        match status_data_rx.try_recv() {
            Ok(m) => {
                trace!("status message: {}", m.to_string());
            }
            Err(_e) => {
                // dbg!(_e);
            }
        }
        sleep(Duration::new(0, 200_000));
    }
    Ok(())
}
