use crate::{
    common::{box_error::BoxError, config::Config, jam_nation_api::JamNationApi, websocket},
    sound::{jack_thread, jam_engine::JamEngine, param_message::ParamMessage},
    utils,
};
use std::{
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};

pub fn run(git_hash: &str) -> Result<(), BoxError> {
    // This is the entry rtjam client
    println!("rtjam client");

    // load up the config to get required info
    let mut config = Config::build();
    config.load_from_file()?;

    let api_url =
        String::from(config.get_value("api_url", "http://rtjam-nation.basscleftech.com/api/1/"));
    let ws_url =
        String::from(config.get_value("ws_url", "ws://rtjam-nation.basscleftech.com/primus"));
    let mac_address = utils::get_my_mac_address()?;

    // Create an api endpoint and register this jamUnit
    let mut api = JamNationApi::new(
        api_url.as_str(),
        "10.10.10.10",
        mac_address.as_str(),
        git_hash,
    );
    // Now loop until we can talk to the mothership
    while !api.has_token() {
        let _register = api.jam_unit_register();
        if !api.has_token() {
            // can't connect to rtjam-nation.  sleep and then keep trying
            sleep(Duration::new(2, 0));
        }
    }

    // Now we have the token, we can pass it to the websocket thread along with the websocket url
    let token = String::from(api.get_token());
    let (to_ws_tx, to_ws_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();
    let (from_ws_tx, from_ws_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();
    let _websocket_handle = thread::spawn(move || {
        let _res = websocket::websocket_thread(&token, &ws_url, from_ws_tx, to_ws_rx);
    });

    // Create channels to/from Jack Engine

    // This is the channel the audio engine will use to send us status data
    let (status_data_tx, status_data_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();

    // This is the channel we will use to send commands to the jack engine
    let (command_tx, command_rx): (mpsc::Sender<ParamMessage>, mpsc::Receiver<ParamMessage>) =
        mpsc::channel();

    let engine = JamEngine::build(status_data_tx, command_rx, api.get_token(), git_hash)?;
    let _jack_thread_handle = thread::spawn(move || {
        let _res = jack_thread::run(engine);
    });

    let _ping_handle = thread::spawn(move || {
        let _res = jam_unit_ping_thread(api);
    });

    // Now this main thread will listen on the mpsc channels
    loop {
        let res = from_ws_rx.try_recv();
        match res {
            Ok(m) => {
                // println!("websocket message: {}", m);
                match ParamMessage::from_json(&m) {
                    Ok(msg) => {
                        // We have a valid param message.  Send it to the jack thread
                        let _res = command_tx.send(msg);
                    }
                    Err(e) => {
                        dbg!(e);
                    }
                }
            }
            Err(_e) => {
                // dbg!(e);
            }
        }
        let res = status_data_rx.try_recv();
        match res {
            Ok(m) => {
                // println!("audio thread message: {}", m.to_string());
                // So we got a message from the jack thread.  See if we need
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

    // let _res = _websocket_handle.join();
    // let _res = _jack_thread_handle.join();
    // let _res = _ping_handle.join();
    // Ok(())
}

fn jam_unit_ping_thread(mut api: JamNationApi) -> Result<(), BoxError> {
    loop {
        while api.has_token() == true {
            // While in this loop, we are going to ping every 10 seconds
            let ping = api.jam_unit_ping()?;
            if ping["jamUnit"].is_null() {
                // Error in the ping.  better re-register
                api.forget_token();
            } else {
                // Successful ping.. Sleep for 10
                sleep(Duration::new(10, 0));
            }
        }
        if !api.has_token() {
            // We need to register the server
            let _register = api.jam_unit_register();
        }
        // This is the timer between registration attempts
        sleep(Duration::new(2, 0));
    }
}
