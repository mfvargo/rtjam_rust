//! Thread used to read/write messages to/from the room abstract in rtjam-nation
use serde_json::Value;

use crate::common::box_error::BoxError;
use crate::common::room::Room;

use crate::common::websock_message::WebsockMessage;
use std::{sync::mpsc, thread::sleep, time::Duration};

/// Used for dependency injection to test init_web_socket and any other theoretical consumers
pub type WebSocketThreadFn = 
    fn(&str, &str, mpsc::Sender<serde_json::Value>, mpsc::Receiver<WebsockMessage>) 
    -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// start a thread with this function.  Pass it the token (name of the room), the
/// websocket uri to use, and two channels.  the first will be used to forward messages
/// received on the websocket to the thread to called us.  The second will be used to
/// read messages from the calling thread that will be written to the room.
pub fn websocket_thread(
    token: &str,                           // Token for the chat room name
    ws_url: &str,                          // URL to connect to the server
    ws_tx: mpsc::Sender<Value>,            // channel to main thread
    ws_rx: mpsc::Receiver<WebsockMessage>, // channel from main thread
) -> Result<(), BoxError> {
    println!("websocket::websocket_thread - Running websocket_thread with token: {}, ws_url: {}", token, ws_url);
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
                                    match msg {
                                        WebsockMessage::Chat(v) => {
                                            if v["context"] == "user" {
                                                // Message from the room, parse and send on to the main thread
                                                match v["message"].as_str() {
                                                    Some(data) => {
                                                        match serde_json::from_str(data) {
                                                            Ok(umsg) => {
                                                                let _res = ws_tx.send(umsg);
                                                            }
                                                            Err(e) => {
                                                                dbg!(data);
                                                                dbg!(e);
                                                            }
                                                        }
                                                    }
                                                    None => {}
                                                }
                                            }
                                        }
                                        WebsockMessage::API(_a, _s) => {
                                            // No need to do anything
                                        }
                                    }
                                }
                                None => {}
                            }

                            // Do we have any messages to send from the main thread?
                            let res = ws_rx.try_recv();
                            match res {
                                Ok(m) => {
                                    // Got a message to send
                                    // println!("sending to room: {}", m);
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
