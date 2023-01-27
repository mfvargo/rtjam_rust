use crate::common::box_error::BoxError;
use crate::common::room::Room;

use std::{sync::mpsc, thread::sleep, time::Duration};

pub fn websocket_thread(
    token: &str,                              // Token for the chat room name
    ws_url: &str,                             // URL to connect to the server
    ws_tx: mpsc::Sender<serde_json::Value>,   // channel to main thread
    ws_rx: mpsc::Receiver<serde_json::Value>, // channel from main thread
) -> Result<(), BoxError> {
    loop {
        match Room::new(token, ws_url) {
            Ok(mut room) => {
                // We have a connected room
                room.join_room(); // Join the chat room
                while room.is_connected() {
                    // We are in the room.  Do our thing
                    // The get_message will block for a few msec before returning.
                    match room.get_message() {
                        Ok(result) => {
                            match result {
                                Some(msg) => {
                                    println!("sock_msg: {}", msg.to_string());
                                    if msg["context"] == "user" {
                                        // Message from the room, send on to the main thread
                                        ws_tx.send(msg)?;
                                    }
                                }
                                None => {}
                            }

                            // Do we have any messages to send from the main thread?
                            let res = ws_rx.try_recv();
                            match res {
                                Ok(m) => {
                                    // Got a message to send
                                    println!("sending to room: {}", m);
                                    room.send_message(&m);
                                }
                                Err(_e) => {
                                    // dbg!(_e);
                                }
                            }
                        }
                        Err(e) => {
                            room.reset();
                            dbg!(e);
                        }
                    }
                }
            }
            Err(e) => {
                dbg!(e);
                // Failed to connect to room.  Wait before trying again
                sleep(Duration::new(2, 0));
            }
        }
    }
}
