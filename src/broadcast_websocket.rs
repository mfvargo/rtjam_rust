use crate::box_error::BoxError;
use json::JsonValue;
use std::{sync::mpsc, thread::sleep, time::Duration};
use tungstenite::{connect, Message};
use url::Url;

pub fn websocket_thread(
    _token: &str,                     // Token for the chat room name
    ws_url: &str,                     // URL to connect to the server
    ws_tx: mpsc::Sender<JsonValue>,   // channel to main thread
    ws_rx: mpsc::Receiver<JsonValue>, // channel from main thread
) -> Result<(), BoxError> {
    let mut room = Room::new();
    loop {
        room.reset();
        let con_result = connect(Url::parse(ws_url).unwrap());
        match con_result {
            Ok(res) => {
                // connect attempt was tried
                let (mut sock, resp) = res;
                dbg!(resp);
                let mut connected = true;
                while connected {
                    let res_msg = sock.read_message();
                    match res_msg {
                        Ok(msg) => {
                            // We got a message from the websocket
                            if msg.is_text() {
                                let (is_ping, pong_msg) = is_primus_ping(&msg);
                                if is_ping {
                                    sock.write_message(Message::Text(pong_msg.into()))?;
                                }
                                // let _msg_res = handle_websocket_message(&mut room, &msg);
                                ws_tx.send(json::parse(msg.to_string().as_str())?)?;
                            }
                        }
                        Err(e) => {
                            dbg!(e);
                            connected = false;
                        }
                    }
                }
            }
            Err(e) => {
                dbg!(e);
            }
        }

        // pause before trying to connect again
        sleep(Duration::new(2, 0));
    }
}

fn is_primus_ping(msg: &Message) -> (bool, String) {
    let msg_body = msg.to_string();
    let is_bool = msg_body.contains("primus::ping::");
    let pong_msg = String::new();
    if is_bool {
        let vec = msg_body.split("::ping::").collect::<Vec<&str>>();
        return (is_bool, format!("\"primus::pong::{}", vec[1]));
    }
    (is_bool, pong_msg)
}

fn handle_websocket_message(room: &mut Room, msg: &Message) -> Result<(), BoxError> {
    let parsed = json::parse(msg.to_string().as_str())?;
    println!("websocket message: {}", parsed.pretty(2));
    // If this is a ping, we send a pong
    match room.get_state() {
        RoomState::Idle => {
            // Send the room join message
            room.join();
        }
        RoomState::Joining => {
            // Is this the response to our join query?
            room.enter();
        }
        RoomState::Inside => {
            // this is a message from somebody in the room
        }
    }
    Ok(())
}

enum RoomState {
    Idle,
    Joining,
    Inside,
}

struct Room {
    state: RoomState,
}
impl Room {
    fn new() -> Self {
        Room {
            state: RoomState::Idle,
        }
    }
    fn join(&mut self) -> () {
        self.state = RoomState::Joining;
    }
    fn enter(&mut self) -> () {
        self.state = RoomState::Inside;
    }
    fn reset(&mut self) -> () {
        self.state = RoomState::Idle;
    }
    fn get_state(&self) -> &RoomState {
        &self.state
    }
}
