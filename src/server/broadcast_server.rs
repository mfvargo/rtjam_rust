//! entry point called by main to run the broadcast component
//!
//! This will create some threads to
//! - listen for audio packets [`crate::common::jam_packet::JamMessage`] and forward them to others
//! - listen for messages from the chatRoom for the audio room being hosted
//! - let the rtjam-nation know this component is registered and alive
use crate::{
    common::{
        box_error::BoxError, config::Config, get_micro_time, jam_nation_api::JamNationApi,
        jam_packet::JamMessage, packet_stream::PacketWriter, stream_time_stat::MicroTimer,
        websock_message::WebsockMessage, websocket, recording::RecordingCatalog,
    },
    server::{
        audio_thread,
        cmd_message::{RoomCommandMessage, RoomParam}, playback_thread,
    },
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
///
/// the original thread that calls run then loops checking mpsc::channels for messages between the websocket
/// and the audio_thread.
pub fn run(git_hash: &str) -> Result<(), BoxError> {
    // This is the entry point for the broadcast server

    // load up the config to get required info
    let mut config = Config::build();
    config.load_from_file()?;

    let api_url = String::from(config.get_value("api_url", "http://rtjam-nation.com/api/1/"));
    let ws_url = String::from(config.get_value("ws_url", "ws://rtjam-nation.com/primus"));
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
        match api.activate_room(port) {
            Ok(res) => {
                if let Some(tok) = res["room"]["token"].as_str() {
                    room_token = tok.to_string();
                }
            }
            Err(e) => {
                dbg!(e);
            }
        }
        if room_token == "" {
            // can't connect to rtjam-nation.  sleep and then keep trying
            sleep(Duration::new(2, 0));
        }
    }
    let at_room_token = room_token.clone();
    let api_room_token = room_token.clone();

    // Let's create a mpsc stream for capturing room output
    let (record_tx, record_rx): (mpsc::Sender<JamMessage>, mpsc::Receiver<JamMessage>) =
        mpsc::channel();
    // Let's create a mpsc stream for playback of room recordings
    let (playback_tx, playback_rx): (mpsc::Sender<JamMessage>, mpsc::Receiver<JamMessage>) =
        mpsc::channel();
    // Let's create a mpsc stream for playback thread commands
    let (playback_cmd_tx, playback_cmd_rx): (mpsc::Sender<RoomCommandMessage>, mpsc::Receiver<RoomCommandMessage>) =
    mpsc::channel();
    // Create playback thread
    let _playback_handle = thread::spawn(move || {
        let _res = playback_thread::run(
            playback_cmd_rx, 
            playback_tx);
    });

    // Now we have the token, we can pass it to the websocket thread along with the websocket url
    let (to_ws_tx, to_ws_rx): (mpsc::Sender<WebsockMessage>, mpsc::Receiver<WebsockMessage>) =
        mpsc::channel();
    let (from_ws_tx, from_ws_rx): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel();
    let _websocket_handle = thread::spawn(move || {
        let _res = websocket::websocket_thread(&room_token, &ws_url, from_ws_tx, to_ws_rx);
    });

    // Create a thread to host the room
    let (audio_tx, audio_rx): (mpsc::Sender<WebsockMessage>, mpsc::Receiver<WebsockMessage>) =
        mpsc::channel();
    let _room_handle = thread::spawn(move || {
        let _res = audio_thread::run(room_port, audio_tx, &at_room_token, record_tx, playback_rx);
    });

    // let _ping_handle = thread::spawn(move || {
    //     let _res = broadcast_ping_thread(api, port);
    // });

    let mut catalog = RecordingCatalog::new("recs")?;
    let mut dmpfile = PacketWriter::new("audio.dmp")?;
    let mut transport_update_timer = MicroTimer::new(get_micro_time(), 333_000);
    let mut ping_timer = MicroTimer::new(get_micro_time(), 10_000_000);  // Every 10 seconds
    // Now this main thread will listen on the mpsc channels
    loop {
        let now_time = get_micro_time();
        let res = from_ws_rx.try_recv();
        match res {
            Ok(m) => {
                // This is where we listen for commands from the room to do stuff.
                println!("websocket message: {}", m.to_string());
                // Reset the transport update timer so that changes to state are sent back
                // immediately instead of on next timer interval.
                transport_update_timer.reset(0);
                // Process the room commmand
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
                        RoomParam::UploadRecording => {
                            // Message asking us to upload a recording to S3
                            println!("recording id: {}", cmd.ivalue_1);
                            match api.get_signed_url(&api_room_token, cmd.ivalue_1 as u32) {
                                Ok(ret) => {dbg!(&ret);}
                                Err(e) => {dbg!(e);}
                            }
                        }
                        _ => {
                            dbg!(&m);
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
        // forward any messages from the audio thead to the websocket (latency updates)
        for m in audio_rx.try_iter() {
            to_ws_tx.send(m)?;
        }
        // drain out any packets that are to be recorded
        for msg in record_rx.try_iter() {
            // got a Jam Message
            // println!("record: {}", msg);
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
            to_ws_tx.send(WebsockMessage::Chat(serde_json::json!({
                "speaker": "RoomChatRobot",
                "transportStatus": dmpfile.get_status(),
            })))?;
            if catalog.is_dirty() {
                println!("updating catalog");
                to_ws_tx.send(WebsockMessage::Chat(serde_json::json!({
                    "speaker": "RoomChatRobot",
                    "listRecordings": catalog.as_json(),
                })))?;
            }
        }
        // This used to be a separate thread.  Just put it in here so we can use the api object
        if ping_timer.expired(now_time) {
            ping_timer.reset(now_time);
            // send a ping to the nation
            match api.broadcast_unit_ping() {
                Ok(_ping) => {
                }
                Err(e) => {
                    dbg!(e);
                }
            }

        }
        // This is the timer between channel polling
        sleep(Duration::new(0, 1_000));
    }

    // Code won't ever get here
    // let _res = room_handle.join();
    // let _res = websocket_handle.join();
    // Ok(())
}