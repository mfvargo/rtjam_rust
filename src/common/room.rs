//! Chat room hosted on rtjam-nation that used to communicate with the u/x
//!
//! (meet me in the middle)  The sound or broadcast components will create a room with the
//! token pass in on the rtjam-nation server.  The u/x will also join this same room
//! allowing them to communicate.
//!
//! The room is used by the websocket thread.  By creating the websocket thread you
//! are creating a room and the full duplex message channel to communicate with it
use crate::common::box_error::BoxError;
use crate::common::websock_message::WebsockMessage;
use serde_json::{json, Value};
use std::{
    net::{TcpStream, ToSocketAddrs},
    thread::sleep,
    time::Duration,
};
use tungstenite::{
    client,
    error::{Error, UrlError},
    stream::{Mode, NoDelay},
    Message, WebSocket,
};
use url::Url;

#[derive(PartialEq)]
enum RoomState {
    Idle,
    Inside,
}

/// Holds the room state and the websocket connection to rtjam-nation
pub struct Room {
    state: RoomState,
    token: String,
    sock: WebSocket<TcpStream>,
    msg_id: u64,
}
impl Room {
    /// Create a new room named by the token variable.  THe url string should point to the rtjam-nation server(s)
    pub fn new(token: &str, url: &str) -> Result<Self, BoxError> {
        let stream = Self::make_stream(url)?;
        let (sock, _resp) = client::client(Url::parse(url).unwrap(), stream)?;

        // let (sock, _resp) = connect(Url::parse(url).unwrap())?;
        Ok(Room {
            state: RoomState::Idle,
            token: String::from(token),
            sock: sock,
            msg_id: 0,
        })
    }
    fn make_stream(url: &str) -> Result<TcpStream, BoxError> {
        let url = reqwest::Url::parse(url)?;
        let request = client::IntoClientRequest::into_client_request(url)?;
        let uri = request.uri();
        let mode = client::uri_mode(uri)?;
        // let host = request.uri().host().unwrap();
        let host = request
            .uri()
            .host()
            .ok_or(Error::Url(UrlError::NoHostName))?;
        let port = uri.port_u16().unwrap_or(match mode {
            Mode::Plain => 80,
            Mode::Tls => 443,
        });
        let addrs = (host, port).to_socket_addrs()?;
        let mut stream = TcpStream::connect(addrs.as_slice())?;
        NoDelay::set_nodelay(&mut stream, true)?;
        stream.set_read_timeout(Some(Duration::new(0, 200_000_000)))?; // poll 5 times per second
        Ok(stream)
    }
    /// once connected via the websocket, you can join the room on the nation server.  This code
    /// will always first try to create the room (for the case where it does not yet exist).  It will
    /// then send the message to join it.
    ///
    /// The room is managed by the actionHero server running on rtjam-nation
    pub fn join_room(&mut self) -> () {
        self.send_message(&WebsockMessage::API(
            "createChatRoom".to_string(),
            json!({"name": self.token.as_str(), "action": "createChatRoom"}),
        ));
        // TODO:  This is a hack.  Sometimes the room needs to get created and we need to wait before trying to join
        // other times, the room is already created and this will kick an harmless "room already exists" error we can ignore.
        // So we will sleep here for 1 second to give the server time to create the room on the first time scenario
        sleep(Duration::new(1, 0));
        let msg = json!({
          "event": "roomAdd",
          "room": self.token.as_str(),
          "messageId": self.msg_id,
        });
        self.msg_id += 1;
        let _res = self.sock.write_message(Message::Text(msg.to_string()));
        self.state = RoomState::Inside;
    }
    /// This is called to clear the room state.  Done when the websocket connection breaks down.
    pub fn reset(&mut self) -> () {
        self.state = RoomState::Idle;
    }
    pub fn is_connected(&self) -> bool {
        self.state == RoomState::Inside
    }

    fn is_primus_ping(&mut self, msg: &Message) -> bool {
        let msg_body = msg.to_string();
        let is_bool = msg_body.contains("primus::ping::");
        if is_bool {
            // Send the pong
            let vec = msg_body.split("::ping::").collect::<Vec<&str>>();
            let _res = self
                .sock
                .write_message(Message::Text(format!("\"primus::pong::{}", vec[1])));
        }
        is_bool
    }

    /// used by websocket thread to send a message to the chat room
    pub fn send_message(&mut self, msg: &WebsockMessage) -> () {
        let mut jmsg = json!({
            "messageId": self.msg_id,
        });
        match msg {
            WebsockMessage::Chat(v) => {
                jmsg["event"] = "say".into();
                jmsg["room"] = self.token.as_str().into();
                jmsg["message"] = v.to_string().into();
            }
            WebsockMessage::API(action, params) => {
                // TODO:  Format this into an API request
                jmsg["event"] = "action".into();
                jmsg["params"] = params.clone();
                jmsg["params"]["action"] = action.as_str().into();
            }
        }
        self.msg_id += 1;
        // println!("sending this message: {}", jmsg.to_string());
        let _res = self.sock.write_message(Message::Text(jmsg.to_string()));
    }

    /// Used by the websocket thread to read any pending messages from the room.
    /// This code filters out primus ping housekeeping messages.  Also any non-user messages
    /// will be filtered out.  this blocks for up to 200msec
    pub fn get_message(&mut self) -> Result<Option<WebsockMessage>, BoxError> {
        match self.sock.read_message() {
            Ok(msg) => {
                // dbg!(&msg);
                if self.is_primus_ping(&msg) {
                    Ok(None)
                } else {
                    // This is not a ping
                    let jvalue: Value = serde_json::from_str(msg.to_string().as_str())?;
                    if jvalue["context"] == "user" {
                        Ok(Some(WebsockMessage::Chat(jvalue)))
                    } else {
                        // println!("non-user message: {}", jvalue.to_string());
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                match e {
                    Error::Io(ioerr) => {
                        if ioerr.kind() == std::io::ErrorKind::WouldBlock
                            || ioerr.kind() == std::io::ErrorKind::TimedOut
                        {
                            // timeout reading the websocket
                            return Ok(None);
                        } else {
                            return Err(ioerr.into());
                        }
                    }
                    _ => return Err(e.into()),
                };
            }
        }
    }
}

#[cfg(test)]
mod test_room {
    use super::*;

    #[test]
    fn can_build() {
        let _room = Room::new("foobar", "ws://rtjam-nation.com/primus");
    }

    #[test]
    fn can_tcp() {
        let _stream = TcpStream::connect("3.101.28.175:80").unwrap();
    }
}
