use json::JsonValue;

use crate::{
    audio_thread, box_error::BoxError, broadcast_websocket, config::Config,
    jam_nation_api::JamNationApi,
};
use std::{
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};

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
    // TODO: figure out way to get lan ip and mac address
    let mut api = JamNationApi::new(api_url.as_str(), "10.10.10.10", "test:mac", git_hash);
    while !api.has_token() {
        let _register = api.broadcast_unit_register();
        // Activate the room
        let _room_activate = api.activate_room(port)?;
        if !api.has_token() {
            // can't connect to rtjam-nation.  sleep and then keep trying
            sleep(Duration::new(2, 0));
        }
    }

    // Now we have the token, we can pass it to the websocket thread along with the websocket url
    let token = String::from(api.get_token());
    let (ws_tx, ws_rx): (mpsc::Sender<JsonValue>, mpsc::Receiver<JsonValue>) = mpsc::channel();
    let _websocket_handle = thread::spawn(move || {
        let _res = broadcast_websocket::websocket_thread(&token, &ws_url, ws_tx);
    });

    // Create a thread to host the room
    let (audio_tx, audio_rx): (mpsc::Sender<JsonValue>, mpsc::Receiver<JsonValue>) =
        mpsc::channel();
    let _room_handle = thread::spawn(move || {
        let _res = audio_thread::run(room_port, audio_tx);
    });

    let _ping_handle = thread::spawn(move || {
        let _res = broadcast_ping_thread(api, port);
    });

    // Now this main thread will listen on the mpsc channels
    loop {
        let res = ws_rx.try_recv();
        match res {
            Ok(m) => {
                println!("websocket message: {}", m.pretty(2));
            }
            Err(e) => {
                // dbg!(e);
            }
        }
        let res = audio_rx.try_recv();
        match res {
            Ok(m) => {
                println!("audio thread message: {}", m.pretty(2));
            }
            Err(e) => {
                // dbg!(e);
            }
        }
        // This is the timer between registration attempts
        sleep(Duration::new(0, 200_000));
    }

    // Code won't ever get here
    // let _res = room_handle.join();
    // let _res = websocket_handle.join();
    // Ok(())
}

fn broadcast_ping_thread(mut api: JamNationApi, port: u32) -> Result<(), BoxError> {
    loop {
        while api.has_token() == true {
            // While in this loop, we are going to ping every 10 seconds
            let ping = api.broadcast_unit_ping()?;
            if ping["broadcastUnit"].is_null() {
                // Error in the ping.  better re-register
                api.forget_token();
            } else {
                // Successful ping.. Sleep for 10
                sleep(Duration::new(10, 0));
            }
        }
        if !api.has_token() {
            // We need to register the server
            let _register = api.broadcast_unit_register();
            // Activate the room
            let _room_activate = api.activate_room(port)?;
        }
        // This is the timer between registration attempts
        sleep(Duration::new(2, 0));
    }
}
