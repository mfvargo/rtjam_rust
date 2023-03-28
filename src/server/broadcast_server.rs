//! entry point called by main to run the broadcast component
//!
//! This will create some threads to
//! - listen for audio packets [`crate::common::jam_packet::JamMessage`] and forward them to others
//! - listen for messages from the chatRoom for the audio room being hosted
//! - let the rtjam-nation know this component is registered and alive
use crate::{
    common::{box_error::BoxError, config::Config, jam_nation_api::JamNationApi, websocket},
    server::audio_thread,
    utils,
};
use std::{
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};

/// To start a broadcast component, call this function
///
/// pass in the git_hash associated with the build so the nation can know what we are running.
///
/// This function will start additional threads.  
/// - websocket thread - creates a websocket connection to rtjam-nation and creates a chatRoom
/// - audio thread - listens for UDP datagrams and forwards to others in the audio room
/// - broadcast ping thread - periodically updates rtjam-nation with keepalives so it knows the room is up
///
/// the original thread that calls run then loops checking mpsc::channels for messages between the websocket
/// and the audio_thread.  The broadcast ping thread just runs by itself (fire and forget)
pub fn run(git_hash: &str) -> Result<(), BoxError> {
    // This is the entry point for the broadcast server

    // load up the config to get required info
    let mut config = Config::build();
    config.load_from_file()?;

    let api_url =
        String::from(config.get_value("api_url", "http://rtjam-nation.basscleftech.com/api/1/"));
    let ws_url =
        String::from(config.get_value("ws_url", "ws://rtjam-nation.basscleftech.com/primus"));
    let port: u32 = config.get_u32_value("port", 7891);
    println!("port: {}", port);
    let room_port = port.clone();
    let mac_address = utils::get_my_mac_address()?;
    let mut room_token = "".to_string();
    // Create an api endpoint and register this server
    // TODO: figure out way to get lan ip and mac address
    let mut api = JamNationApi::new(api_url.as_str(), mac_address.as_str(), git_hash);
    while room_token == "" {
        let _register = api.broadcast_unit_register();
        // Activate the room
        let room_activate = api.activate_room(port)?;
        if let Some(tok) = room_activate["room"]["token"].as_str() {
            room_token = tok.to_string();
        }
        if !api.has_token() {
            // can't connect to rtjam-nation.  sleep and then keep trying
            sleep(Duration::new(2, 0));
        }
    }

    // Now we have the token, we can pass it to the websocket thread along with the websocket url
    let (to_ws_tx, to_ws_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();
    let (from_ws_tx, from_ws_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();
    let _websocket_handle = thread::spawn(move || {
        let _res = websocket::websocket_thread(&room_token, &ws_url, from_ws_tx, to_ws_rx);
    });

    // Create a thread to host the room
    let (audio_tx, audio_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();
    let _room_handle = thread::spawn(move || {
        let _res = audio_thread::run(room_port, audio_tx);
    });

    let _ping_handle = thread::spawn(move || {
        let _res = broadcast_ping_thread(api, port);
    });

    // Now this main thread will listen on the mpsc channels
    loop {
        let res = from_ws_rx.try_recv();
        match res {
            Ok(m) => {
                println!("websocket message: {}", m.to_string());
            }
            Err(_e) => {
                // dbg!(e);
            }
        }
        let res = audio_rx.try_recv();
        match res {
            Ok(m) => {
                // println!("audio thread message: {}", m.to_string());
                // So we got a message from the audio thread.  See if we need
                // To pass this along to the websocket
                to_ws_tx.send(m)?;
            }
            Err(_e) => {
                // dbg!(_e);
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
