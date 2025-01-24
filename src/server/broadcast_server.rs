//! entry point called by main to run the broadcast component
//!
//! This will create some threads to
//! - listen for audio packets [`crate::common::jam_packet::JamMessage`] and forward them to others
//! - listen for messages from the chatRoom for the audio room being hosted
//! - let the rtjam-nation know this component is registered and alive
use crate::{
    common::{
        box_error::BoxError, 
        config::Config, 
        get_micro_time, 
        jam_nation_api::JamNationApi, 
        jam_packet::JamMessage, 
        packet_stream::PacketWriter, 
        recording::RecordingCatalog,
        stream_time_stat::MicroTimer, 
        websock_message::WebsockMessage, 
        websocket
    },
    server::{
        audio_thread,
        cmd_message::{RoomCommandMessage, RoomParam},
        ping_thread::broadcast_ping_thread, 
        playback_thread
    },
    utils,
};
use std::{
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};
use chrono::{Local, NaiveTime, SubsecRound};
use log::{debug, error, info, trace, warn};

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
    info!("Starting run function");

    // load up the config to get required info
    // TODO: keep bubbling up the config file name to remove hard coding, and make configurable for testing fun and games
    // NOTE: Is this the best place for these defaults, or should the be either encapsulated in the config object, or passed in as arguments?
    let defaults = json::object! {
        "api_url": "http://rtjam-nation.com/api/1/",
        "ws_url": "ws://rtjam-nation.com/primus",
        "room_mode": "separate",
        "port": 7891,
    };
    let config = Config::build(String::from("settings.json"), defaults);
    let config = match config {
        Ok(c) => c,
        Err(e) => {
            error!("Issue with config file or parameter: {}", e);
            return Err(Box::new(e));
        }
    };

    let api_url = String::from(config.get_str_value("api_url", None)?);
    let ws_url = String::from(config.get_str_value("ws_url", None)?);
    let room_mode = config.get_str_value("room_mode", None)? == "mix";
    let port: u32 = config.get_u32_value("port", None)?;
    let room_port = port.clone();
    let mac_address = utils::get_my_mac_address()?;
    let mut room_token = "".to_string();
    // Create an api endpoint and register this server
    // TODO: figure out way to get lan ip and mac address
    let mut api = JamNationApi::new(&api_url, &mac_address, &String::from(git_hash));
    while room_token == "" {
        let _register = api.broadcast_unit_register();
        // Activate the room
        match api.activate_room(port) {
            Ok(res) => {
                if let Some(tok) = res["room"]["token"].as_str() {
                    room_token = tok.to_string();
                }
            }
            Err(e) => {
                warn!("{}", e);
            }
        }
        if room_token == "" {
            // can't connect to rtjam-nation.  sleep and then keep trying
            sleep(Duration::new(2, 0));
        }
    }
    let at_room_token = room_token.clone();

    // Let's create a mpsc channel to send messages to the websocket
    let (to_ws_tx, to_ws_rx): (mpsc::Sender<WebsockMessage>, mpsc::Receiver<WebsockMessage>) =
        mpsc::channel();
        
    // Let's create a mpsc stream for capturing room output
    let (record_tx, record_rx): (mpsc::Sender<JamMessage>, mpsc::Receiver<JamMessage>) =
        mpsc::channel();
    // Let's create a mpsc stream for playback of room recordings
    let (playback_tx, playback_rx): (mpsc::Sender<JamMessage>, mpsc::Receiver<JamMessage>) =
        mpsc::channel();
    // Let's create a mpsc stream for playback thread commands
    let (playback_cmd_tx, playback_cmd_rx): (mpsc::Sender<RoomCommandMessage>, mpsc::Receiver<RoomCommandMessage>) =
    mpsc::channel();
    // Clone the ws_tx channel so the playback thread can send playback status messages
    let pback_ws_tx = to_ws_tx.clone();
    // Create playback thread
    let _playback_handle = thread::spawn(move || {
        let _res = playback_thread::run(
            pback_ws_tx,
            playback_cmd_rx, 
            playback_tx);
    });

    // Now we have the token, we can pass it to the websocket thread along with the websocket url
    let (from_ws_tx, from_ws_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();
    let _websocket_handle = thread::spawn(move || {
        let _res = websocket::websocket_thread(&room_token, &ws_url, from_ws_tx, to_ws_rx);
    });

    // create a command channel to the audio thread 
    let (audio_cmd_tx, audio_cmd_rx): (mpsc::Sender<RoomCommandMessage>, mpsc::Receiver<RoomCommandMessage>) =
    mpsc::channel();

    // Clone the websocket channel tx so the audio thread can send to it too.
    let audio_tx = to_ws_tx.clone();
    let _room_handle = thread::spawn(move || {
        let _res = audio_thread::run(
            room_port, 
            audio_cmd_rx,
            audio_tx, 
            &at_room_token, 
            record_tx, 
            playback_rx, 
            room_mode);
    });

    let _ping_handle = thread::spawn(move || {
        let _res = broadcast_ping_thread(api, port);
    });

    let mut catalog = RecordingCatalog::new("recs")?;
    let mut dmpfile = PacketWriter::new("audio.dmp")?;
    let mut transport_update_timer = MicroTimer::new(get_micro_time(), 333_000);
    // Now this main thread will listen on the mpsc channels
    loop {
        let now_time = get_micro_time();
        let res = from_ws_rx.try_recv();
        match res {
            Ok(m) => {
                // This is where we listen for commands from the room to do stuff.
                debug!("websocket message: {}", m.to_string());
                transport_update_timer.reset(0);
                match RoomCommandMessage::from_json(&m) {
                    Ok(mut cmd) => match cmd.param {
                        RoomParam::Record => {
                            dmpfile.is_writing = true;
                        }
                        RoomParam::Stop => {
                            dmpfile.is_writing = false;
                            catalog.load_recordings()?;
                            playback_cmd_tx.send(cmd)?;
                        }
                        RoomParam::ListFiles => {
                            catalog.load_recordings()?;
                        }
                        RoomParam::SaveRecording => {
                            // Copy audio.dmp into the catalog
                            catalog.add_file("audio.dmp");
                            dmpfile = PacketWriter::new("audio.dmp")?;
                        }
                        RoomParam::DeleteRecording => {
                            if cmd.svalue == "" {
                                dmpfile = PacketWriter::new("audio.dmp")?;
                            } else {
                                catalog.delete_file(&cmd.svalue);
                            }
                        }
                        RoomParam::Play => {
                            if cmd.svalue == "" {
                                cmd.svalue = "../audio.dmp".to_string();
                            }
                            playback_cmd_tx.send(cmd)?;
                        }
                        RoomParam::Seek => {
                            playback_cmd_tx.send(cmd)?;
                        }
                        _ => {
                            audio_cmd_tx.send(cmd)?;
                        }
                    },
                    Err(e) => {
                        dbg!(e);
                    }
                }
            }
            Err(_e) => {
                // dbg!(e);
            }
        }
        // drain out any recording audio
        for msg in record_rx.try_iter() {
            // got a Jam Message
            match dmpfile.write_message(&msg) {
                Ok(_) => (),
                Err(e) => {
                    dbg!(e);
                }
            }
        }
        if transport_update_timer.expired(now_time) {
            transport_update_timer.reset(now_time);
            // send transport update
            trace!("transport status {}", dmpfile.get_status());
            to_ws_tx.send(WebsockMessage::Chat(serde_json::json!({
                "speaker": "RoomChatRobot",
                "transportStatus": dmpfile.get_status(),
            })))?;
            if catalog.is_dirty() {
                debug!("updating recording catalog {}", catalog.as_json());
                to_ws_tx.send(WebsockMessage::Chat(serde_json::json!({
                    "speaker": "RoomChatRobot",
                    "listRecordings": catalog.as_json(),
                })))?;
            }
        }
        // This is the timer between channel polling
        if restart_time() {
            std::process::exit(-1);
        }
        sleep(Duration::new(0, 500_000));
    }

    // Code won't ever get here
    // let _res = room_handle.join();
    // let _res = websocket_handle.join();
    // Ok(())
}

fn restart_time() -> bool {
    let now = Local::now();
    let current_time = now.time().round_subsecs(0);
    if let Some(wee_hours) = NaiveTime::from_hms_opt(2, 07, 08) {
        return current_time == wee_hours;
    }
    false
}
