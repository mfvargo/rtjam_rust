use crate::{
    box_error::BoxError,
    config::Config,
    jam_nation_api::JamNationApi,
    jam_packet::JamMessage,
    player_list::{get_micro_time, PlayerList},
};
use std::{
    io::ErrorKind,
    net::UdpSocket,
    thread::{self, sleep},
    time::Duration,
};
use tungstenite::{connect, Message};
use url::Url;

pub fn run(git_hash: &str) -> Result<(), BoxError> {
    // This is the entry point for the broadcast server

    // load up the config to get required info
    let mut config = Config::build();
    config.load_from_file()?;

    let api_url =
        String::from(config.get_value("api_url", "http://rtjam-nation.basscleftech.com/api/1/"));
    let ws_url =
        String::from(config.get_value("ws_url", "ws://rtjam-nation.basscleftech.com/primus"));
    let port: u32 = config.get_value("port", "7891").parse()?;
    let room_port = port.clone();

    // Create an api endpoint and register this server
    let mut api = JamNationApi::new(api_url.as_str(), "10.10.10.10", "test:mac", git_hash);
    while !api.has_token() {
        let _register = api.broadcast_unit_register();
        // Activate the room
        let _room_activate = api.activate_room(port)?;
        if !api.has_token() {
            sleep(Duration::new(2, 0));
        }
    }

    // Now we have the token, we can pass it to the websocket thread
    let token = String::from(api.get_token());
    let _websocket_handle = thread::spawn(move || {
        let _res = websocket_thread(&token, &ws_url);
    });

    // Create a thread to host the room
    let _room_handle = thread::spawn(move || {
        let _res = audio_thread(room_port);
    });

    // Now the main thread will run the broadcast keepalive code
    loop {
        while api.has_token() == true {
            // While in this loop, we are going to ping every 10 seconds
            let ping = api.broadcast_unit_ping()?;
            // println!("ping: {}", ping.pretty(2));
            if ping["broadcastUnit"].is_null() {
                // Error in the ping.  better re-register
                api.forget_token();
            } else {
                // Successful ping.. Sleep for 10
                // println!("ping!");
                sleep(Duration::new(10, 0));
            }
        }
        if !api.has_token() {
            // We need to register the server
            let _register = api.broadcast_unit_register();
            // Activate the room
            let _room_activate = api.activate_room(port)?;
            // println!("roomActivate: {}", _room_activate.pretty(2));
        }
        // This is the timer between registration attempts
        sleep(Duration::new(2, 0));
    }

    // Code won't ever get here
    // let _res = room_handle.join();
    // let _res = websocket_handle.join();
    // Ok(())
}

fn websocket_thread(_token: &str, ws_url: &str) -> Result<(), BoxError> {
    loop {
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
                                handle_websocket_message(&msg);
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

fn handle_websocket_message(msg: &Message) -> () {
    dbg!(msg);
}

fn audio_thread(port: u32) -> Result<(), BoxError> {
    // So let's create a UDP socket and listen for shit
    let sock = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    sock.set_read_timeout(Some(Duration::new(1, 0)))?;
    let mut players = PlayerList::build();
    let mut msg = JamMessage::build();
    let mut cnt: u64 = 0;

    loop {
        cnt += 1;
        let res = sock.recv_from(msg.get_buffer());
        // get a timestamp to use
        let now_time = get_micro_time();
        // update the player list
        players.prune(now_time);
        match res {
            Ok(r) => {
                let (amt, src) = r;
                if cnt % 1000 == 0 {
                    println!("got {} bytes from {}", amt, src);
                    println!("player: {}", players);
                }
                // check if the packet was good
                if amt <= 0 || !msg.is_valid(amt) || !players.is_allowed(msg.get_client_id()) {
                    continue;
                }
                // set the server timestamp
                msg.set_server_time(now_time.try_into()?);
                // Update this player with the current time
                players.update_player(now_time, msg.get_client_id(), src);
                for player in players.get_players() {
                    if player.address != src {
                        // don't send echo back
                        // send the packet
                        sock.send_to(&msg.get_buffer()[0..amt], player.address)?;
                    }
                }
            }
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => {}
                other_error => {
                    panic!("my socket went nuts! {}", other_error);
                }
            },
        }
    }
}
