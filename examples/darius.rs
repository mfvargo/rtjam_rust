use rtjam_rust::common::box_error::BoxError;
use rtjam_rust::common::websocket::websocket_thread;
use std::{
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};

fn main() -> Result<(), BoxError> {
    // most simple app to test websocket

    let (_to_ws_tx, to_ws_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();
    let (from_ws_tx, from_ws_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();
    let _websocket_handle = thread::spawn(move || {
        let _res = websocket_thread(
            "darius_room",
            "ws://rtjam-nation.com/primus",
            from_ws_tx,
            to_ws_rx,
        );
    });

    loop {
        let res = from_ws_rx.try_recv();
        match res {
            Ok(m) => {
                println!("websocket message: {}", m);
            }
            Err(_e) => {
                // dbg!(e);
            }
        }
        // This is the timer between read attempts
        sleep(Duration::new(1, 0));
    }
}
