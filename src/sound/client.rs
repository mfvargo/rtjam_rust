//! top level entry point called by main to run the sound component
//!
//! This function will never return (except on panic) and will start all the pieces to
//! run the rtjam_sound component.
//!
//! It will first use the [`JamNationApi`] to register the component.  This will return a token
//! from the rtjam-nation.  This token will then be passed to the [`websocket::websocket_thread`] so it
//! can create a room on the rtjam-nation server to be used to communicate with the U/X
//!
//! The same token will be passed to a [`JamEngine`] object that will be moved into the
//! [`jack_thread::run`] function.  The jack_thread code will connect jack inputs and outputs and
//! use the JamEngine to process the audio data.
//!
//! A final thread will be started to wake up every 10 seconds to ping the rtjam-nation server to
//! indicate the component is still alive.
//!
//! Initial thread will then loop relaying mpsc messages between the various threads.
//!
//! All threads and components will return to a reconnect mode in the case that they cannot talk to their
//! necessary systems.  If rtjam-nation goes down for some reason, the websocket will go into
//! reconnect loop till it comes back.  Likewise the ping thread will go into a loop, re-register, and then
//! continue the ping loop. Lastly the jack thread will loop if it comes up before the jack system has
//! started.  Once it can connect to jack for audio, it will continue.
//!
//! TODO:  jack_thread does not recover from jack being stopped after it's already running.  Need to
//! have it re-initialize into acquire more if jack falls down in the middle.
use crate::{
    common::{
        box_error::BoxError, config::Config, get_micro_time, jam_nation_api::JamNationApi,
        stream_time_stat::MicroTimer, websock_message::WebsockMessage, websocket,
    }, hw_control::{hw_control_thread::hw_control_thread, status_light::{has_lights, LightMessage}}, pedals::pedal_board::PedalBoard, sound::{
        // jack_thread,
        jam_engine::JamEngine,
        param_message::{JamParam, ParamMessage},
    }, utils,
};
use serde_json::json;
use thread_priority::{ThreadBuilder, ThreadPriority};
use std::{
    io::{ErrorKind, Write},
    process::Command,
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};
//use log::{debug, info, warn, error, trace, log_enabled, Level};
use log::{trace, debug, info, warn,};

use super::alsa_thread;

/// This is the entry for rtjam client
/// call this from the main function to start the whole thing running.
///
/// note the git_hash string allows the software to tell rtjam-nation what version of code it
/// is currently running.
pub fn run(git_hash: &str, in_dev: String, out_dev: String) -> Result<(), BoxError> {
    // This is the entry rtjam client

    // load up the config to get required info
    // TODO: keep bubbling up the config file name to remove hard coding, and make configurable for testing fun and games
    let mut config = Config::build(String::from("settings.json"));
    config.load_from_file()?;

    let api_url = String::from(config.get_value("api_url", "http://rtjam-nation.com/api/1/"));
    let ws_url = String::from(config.get_value("ws_url", "ws://rtjam-nation.com/primus"));
    let mac_address = utils::get_my_mac_address()?;
    let no_loopback = config.get_bool_value("no_loopback", false);
    debug!("Config values: api_url: {}, ws_url: {}, mac_address: {}, no_loopback: {}", api_url, ws_url, mac_address, no_loopback);

    // Create an api endpoint and register this jamUnit
    let mut api = JamNationApi::new(api_url.as_str(), mac_address.as_str(), git_hash);
    // Now loop until we can talk to the mothership
    while !api.has_token() {
        let _register = api.jam_unit_register();
        if !api.has_token() {
            // can't connect to rtjam-nation.  sleep and then keep trying
            info!("Can't connect to rtjam-nation.  Sleeping 2 seconds the retrying");
            sleep(Duration::new(2, 0));
        }
    }

    // Now we have the token, we can pass it to the websocket thread along with the websocket url
    let token = String::from(api.get_token());
    debug!("API token: {}", token);
    let (to_ws_tx, to_ws_rx): (mpsc::Sender<WebsockMessage>, mpsc::Receiver<WebsockMessage>) =
        mpsc::channel();
    let (from_ws_tx, from_ws_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();
    let _websocket_handle = thread::spawn(move || {
        let _res = websocket::websocket_thread(&token, &ws_url, from_ws_tx, to_ws_rx);
    });

    let mut light_option: Option<mpsc::Sender<LightMessage>> = None;
    // Lets create a thread to control custom hardware
    let (lights_tx, lights_rx): (mpsc::Sender<LightMessage>, mpsc::Receiver<LightMessage>) =
    mpsc::channel();

    if has_lights() {
        light_option = Some(lights_tx);

        let _hw_handle = thread::spawn(move || {
            let _res = hw_control_thread(lights_rx);
        });
    }

    // Create channels to/from Jack Engine

    // This is the channel the audio engine will use to send us status data
    let (status_data_tx, status_data_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();

    // This is the channel we will use to send commands to the jack engine
    let (command_tx, command_rx): (mpsc::Sender<ParamMessage>, mpsc::Receiver<ParamMessage>) =
        mpsc::channel();

    let (pedal_tx, pedal_rx): (mpsc::Sender<PedalBoard>, mpsc::Receiver<PedalBoard>) =
        mpsc::channel();

    if no_loopback {
        info!("Local loopback disabled");
    }
    let engine = JamEngine::new(light_option, status_data_tx, command_rx, pedal_rx, api.get_token(), git_hash, no_loopback)?;
    // let _jack_thread_handle = thread::spawn(move || {
    //     let _res = jack_thread::run(engine);
    // });
    // let _alsa_thread_handle = thread::spawn(move || {
    //     let _res = alsa_thread::run(engine);
    // });

    let builder = ThreadBuilder::default()
        .name("Real-Time Thread".to_string())
        .priority(ThreadPriority::Max);

    let _alsa_handle = builder.spawn(move |_result| {
        match alsa_thread::run(engine, &in_dev, &out_dev) {
            Ok(()) => {
            println!("alsa ended with OK");
            }
            Err(e) => {
                println!("alsa exited with error {}", e);
            }
        }
    })?;

    let _ping_handle = thread::spawn(move || {
        let _res = jam_unit_ping_thread(api);
    });

    // create a timer to ping the websocket to let them know we are here
    let mut websock_room_ping = MicroTimer::new(get_micro_time(), 2_000_000);
    // Now this main thread will listen on the mpsc channels
    loop {
        // This is how you would send a message to the hw control thread!
        // let _res = lights_tx.send(json!({"msg": "hello"}));
        let res = from_ws_rx.try_recv();
        match res {
            Ok(m) => {
                debug!("websocket message: {}", m);
                match ParamMessage::from_json(&m) {
                    Ok(msg) => {
                        match msg.param {
                            // Is this a message we handle right here?
                            JamParam::SetAudioInput => {
                                // client command
                                info!("Set audio input: {}", msg);
                                write_string_to_file("soundin.cfg", &msg.svalue);
                            }
                            JamParam::SetAudioOutput => {
                                // another client command
                                info!("Set audio otuput: {}", msg);
                                write_string_to_file("soundout.cfg", &msg.svalue);
                            }
                            JamParam::ListAudioConfig => {
                                // run aplay -L and send back result
                                let _res = to_ws_tx.send(WebsockMessage::Chat(make_audio_config()));
                            }
                            JamParam::CheckForUpdate => {
                                // See if we need to update ourself
                                // if we just exit, it should check for update on restart
                                info!("Check for update requested. Restarting.");
                                std::process::exit(-1);
                            }
                            JamParam::RandomCommand => {
                                info!("Rando: {}", msg);
                                info!("Output: {}", run_a_command(&msg.svalue));
                                let _res =
                                    to_ws_tx.send(WebsockMessage::Chat(run_a_command(&msg.svalue)));
                            }
                            JamParam::ShutdownDevice => {
                                info!("Exiting app");
                                std::process::exit(-1);
                            }
                            JamParam::LoadBoard => {
                                let idx = msg.ivalue_1 as usize;
                                if idx < 2 {
                                    // Build a pedalboard and send it to the jack thread
                                    let mut board = PedalBoard::new(idx);
                                    board.load_from_json(&msg.svalue);
                                    let _res = pedal_tx.send(board);
                                }
                            }                
                            // This message is for the jamEngine to handle
                            _ => {
                                let _res = command_tx.send(msg);
                            }
                        }
                    }
                    Err(err) => {
                        warn!("JSON parse Error: {}", err);
                    }
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                // Expected result, just means there was no message
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                warn!("websocket: disconnected channel");
            }
        }
        let res = status_data_rx.try_recv();
        match res {
            Ok(m) => {
                // audio messages are noisy, so restrict to the most verbose level
                trace!("audio thread message: {}", m.to_string());
                // So we got a message from the jack thread.  See if we need
                // To pass this along to the websocket
                to_ws_tx.send(WebsockMessage::Chat(m))?;
            }
            Err(mpsc::TryRecvError::Empty) => {
                // Expected result, just means there was no message
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                warn!("audio thread: disconnected channel");
            }
        }
        // Ping room occasionally
        let now = get_micro_time();
        if websock_room_ping.expired(now) {
            to_ws_tx.send(WebsockMessage::Chat(
                json!({"speaker": "UnitChatRobot", "websockPing": {"isRust": true}}),
            ))?;
            websock_room_ping.reset(now);
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
            match api.jam_unit_ping() {
                Ok(ping) => {
                    if ping["jamUnit"].is_null() {
                        // Error in the ping.  better re-register
                        api.forget_token();
                    } else {
                        // Successful ping.. Sleep for 10
                        sleep(Duration::new(10, 0));
                    }
                }
                Err(e) => {
                    api.forget_token();
                    debug!("jam_unit_ping Error: {}", e);
                }
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

fn make_audio_config() -> serde_json::Value {
    let mut driver = String::from("hw:USB");
    let mut cards = String::from("");
    match Command::new("cat").arg("soundin.cfg").output() {
        Ok(dev) => {
            driver = String::from_utf8_lossy(&dev.stdout).to_string();
        }
        Err(err) => {
            debug!("audio config Error: {}", err);
        }
    }
    match Command::new("aplay").arg("-l").output() {
        Ok(output) => {
            cards = String::from_utf8_lossy(&output.stdout).to_string();
        }
        Err(err) => {
            debug!("aplay Error: {}", err);
        }
    }
    json!({
        "audioHardware": {
            "driver": driver,
            "cards": cards,
        },
        "speaker": "UnitChatRobot"
    })
}

fn run_a_command(cmd_line: &str) -> serde_json::Value {
    let vals = cmd_line.split(" ").collect::<Vec<&str>>();
    let mut rval = String::from("Error");
    let mut cmd = Command::new(vals[0]);
    for arg in &vals[1..] {
        cmd.arg(arg);
    }
    match cmd.output() {
        Ok(output) => {
            rval = String::from_utf8_lossy(&output.stdout).to_string();
        }
        Err(err) => {
            debug!("run command Error: {}, command: {}", err, cmd_line);
        }
    }
    json!({
        "cmdOutput": rval,
        "speaker": "UnitChatRobot"
    })
}

fn write_string_to_file(fname: &str, contents: &str) -> () {
    match std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(fname)
    {
        Ok(mut f) => {
            let _res = f.write_all(contents.as_bytes());
            let _res = f.sync_all();
        }
        Err(error) => {
            // File open failed.  See if we need to create it
            match error.kind() {
                ErrorKind::NotFound => {
                    // no file, create one
                    match std::fs::File::create(fname) {
                        Ok(mut f) => {
                            let _res = f.write_all(contents.as_bytes());
                            let _res = f.sync_all();
                        }
                        Err(err) => {
                            warn!("create file Error: {}", err);
                        }
                    }
                }
                other_error => {
                    debug!("Unexpected Error type writing to file: {}", other_error);
                    warn!("Failed to create file: {}", fname);
                }
            }
        }
    }
}
