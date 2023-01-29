use crate::common::box_error::BoxError;
use serde_json::{json, Value};
use std::{
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    thread::sleep,
    time::Duration,
};
use tungstenite::{
    client,
    error::{Error, UrlError},
    http::Uri,
    stream::{Mode, NoDelay},
    Message, WebSocket,
};
use url::Url;

#[derive(PartialEq)]
enum RoomState {
    Idle,
    Inside,
}

pub struct Room {
    state: RoomState,
    token: String,
    sock: WebSocket<TcpStream>,
    msg_id: u64,
}
impl Room {
    pub fn new(toke: &str, url: &str) -> Result<Self, BoxError> {
        let stream = Self::make_stream(url)?;
        let (sock, _resp) = client::client(Url::parse(url).unwrap(), stream)?;

        // let (sock, _resp) = connect(Url::parse(url).unwrap())?;
        Ok(Room {
            state: RoomState::Idle,
            token: String::from(toke),
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
        let mut stream = Self::connect_to_some(addrs.as_slice(), request.uri())?;
        NoDelay::set_nodelay(&mut stream, true)?;
        stream.set_read_timeout(Some(Duration::new(0, 200_000_000)))?; // poll 5 times per second
        Ok(stream)
    }
    fn connect_to_some(addrs: &[SocketAddr], uri: &Uri) -> Result<TcpStream, Error> {
        for addr in addrs {
            dbg!("Trying to contact {} at {}...", uri, addr);
            if let Ok(stream) = TcpStream::connect(addr) {
                return Ok(stream);
            }
        }
        Err(Error::Url(UrlError::UnableToConnect(uri.to_string())))
    }
    pub fn join_room(&mut self) -> () {
        let msg = json!({
          "event": "action",
          "params": {
            "name": self.token.as_str(),
            "action": "createChatRoom",
          },
          "messageId": self.msg_id,
        });
        let _res = self.sock.write_message(Message::Text(msg.to_string()));

        // TODO:  This is a hack.  Sometimes the room needs to get created and we need to wait before trying to join
        // other times, the room is already created and this will kick an harmless "room already exists" error we can ignore.
        // So we will sleep here for 1 second to give the server time to create the room on the first time scenario
        sleep(Duration::new(1, 0));
        self.msg_id += 1;
        let msg = json!({
          "event": "roomAdd",
          "room": self.token.as_str(),
          "messageId": self.msg_id,
        });
        self.msg_id += 1;
        let _res = self.sock.write_message(Message::Text(msg.to_string()));
        self.state = RoomState::Inside;
    }
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

    pub fn send_message(&mut self, msg: &Value) -> () {
        let msg = json!({
            "event": "say",
            "room": self.token.as_str(),
            "message": msg.to_string(),
            "messageId": self.msg_id,
        });
        self.msg_id += 1;
        println!("sending this message: {}", msg.to_string());
        let _res = self.sock.write_message(Message::Text(msg.to_string()));
    }

    pub fn get_message(&mut self) -> Result<Option<Value>, BoxError> {
        match self.sock.read_message() {
            Ok(msg) => {
                if self.is_primus_ping(&msg) {
                    Ok(None)
                } else {
                    // This is not a ping
                    let jvalue: Value = serde_json::from_str(msg.to_string().as_str())?;
                    if jvalue["context"] == "user" {
                        Ok(Some(jvalue))
                    } else {
                        // println!("non-user message: {}", jvalue.to_string());
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                match e {
                    Error::Io(ioerr) => {
                        if ioerr.kind() == std::io::ErrorKind::WouldBlock {
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
