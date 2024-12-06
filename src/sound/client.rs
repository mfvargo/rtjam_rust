//! top level entry point called by main to run the sound component
//!
//! This function will never return (except on panic) and will start all the pieces to
//! run the rtjam_sound component.
//!
//! It will first use the [`JamNationApi`] to register the component.  This will return a token
//! from the rtjam-nation.  This token will then be passed to the [`websocket_thread`] so it
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
//! have it re-initialize into acquire mode if jack falls down in the middle.
use crate::{
    common::{
        box_error::BoxError,
        config::Config,
        get_micro_time,
        jam_nation_api::{JamNationApi,JamNationApiTrait},
        stream_time_stat::MicroTimer,
        websock_message::WebsockMessage, 
        websocket::{websocket_thread, WebSocketThreadFn},
    }, 
    hw_control::{
        hw_control_thread::hw_control_thread, 
        status_light::{has_lights, LightMessage},
    }, 
    pedals::pedal_board::PedalBoard, 
    sound::{
        jack_thread,
        alsa_thread,
        jam_engine::JamEngine,
        param_message::{JamParam, ParamMessage},
    }, 
    utils,
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
use log::{trace, debug, info, warn, error};

/// This is the entry point for the rtjam client.
///
/// Call this function from the main function to start the entire sound component running.
///
/// # Parameters
///
/// - `git_hash`: A `String` representing the current version of the code, which will be sent to the rtjam-nation server.
/// - `in_dev`: A `String` specifying the input device to be used for audio processing.
/// - `out_dev`: A `String` specifying the output device for audio playback.
///
/// note the git_hash string allows the software to tell rtjam-nation what version of code it
/// is currently running.
pub fn run(
    git_hash: String, 
    use_alsa: bool, 
    in_dev: String, 
    out_dev: String
) -> Result<(), BoxError> {
    info!("client - starting run function");
    // Initialize config and API connection
    // TODO: pass in the config file name as an optional parameter
    let (api_url, ws_url, mac_address, no_loopback) = init_config(None)?;
    debug!("client::run - config file init complete");

    let mut api = JamNationApi::new(&api_url, &mac_address, &git_hash);
    let _connected_api = init_api_connection(&mut api)?;
    let token = String::from(api.get_token());
    debug!("client::run - API connection established. Token: {}", token);

    // Initialize websocket channels and thread
    let (to_ws_tx, from_ws_rx, _ws_handle) = init_websocket_thread(&token, &ws_url, None)?;
    debug!("client::run - websocket connection established");

    // Initialize hardware control channels and thread if needed
    let (light_option, _hw_handle) = init_hardware_control()?;
    debug!("client::run - hardware control established");

    // Initialize audio engine channels
    let (status_data_tx, status_data_rx) = mpsc::channel();
    let (command_tx, command_rx) = mpsc::channel();
    let (pedal_tx, pedal_rx) = mpsc::channel();

    // Create and start audio engine
    let engine = JamEngine::new(
        light_option,
        status_data_tx,
        command_rx,
        pedal_rx,
        api.get_token(),
        git_hash.as_str(),
        no_loopback,
    )?;
    debug!("client::run - audio engine started");

    // Start appropriate hardware level sound thread
    if use_alsa {
        info!("client - using ALSA");
        let _sound_handle = start_alsa_thread(engine, &in_dev, &out_dev)?;
        debug!("client::run - ALSA thread started");
    } else {
        info!("client - using Jack");
        let _sound_handle = start_jack_thread(engine)?;
        debug!("client::run - Jack thread started");        
    }

    // Start ping thread
    let _ping_handle = thread::spawn(move || {
        let _res = jam_unit_ping_thread(&mut api);
    });
    debug!("client::run - ping handle started");

    debug!("client::run - setup complete, beginning main event loop");
    run_main_loop(from_ws_rx, to_ws_tx, command_tx, pedal_tx, status_data_rx)?;

    Ok(())
}

/// Wraps client specific config value extraction into a convenience function.
/// 
/// This function attempts to load configuration values from a specified file. 
/// If no file is provided, it defaults to "settings.json". It returns a 
/// tuple containing the following values:
/// 
/// - `api_url`: The URL for the API endpoint.
/// - `ws_url`: The WebSocket URL for real-time communication.
/// - `mac_address`: The MAC address of the device.
/// - `no_loopback`: A boolean indicating whether local loopback is disabled.
/// 
/// # Errors
/// This function will return an error if the configuration file cannot be 
/// read or if any of the expected values are missing or invalid. 
fn init_config(config_file: Option<&str>) -> Result<(String, String, String, bool), BoxError> {
    let default_params = json::object! {
        "api_url": "http://rtjam-nation.com/api/1/",
        "ws_url": "ws://rtjam-nation.com/primus",
        "no_loopback": false
    };

    // Default to settings.json if no file is provided
    let filename = config_file.unwrap_or("settings.json");
    info!("Using config file: {}", filename);

    let config = Config::build(String::from(filename), default_params)
        .map_err(|e| {
            error!("Issue with config file or parameter: {}", e);
            e
        })?;

    let api_url = String::from(config.get_str_value("api_url", None)?);
    let ws_url = String::from(config.get_str_value("ws_url", None)?);
    let mac_address = utils::get_my_mac_address()?;
    let no_loopback = config.get_bool_value("no_loopback", None)?;

    info!(
        "Config values: api_url: {}, ws_url: {}, mac_address: {}, no_loopback: {}",
        api_url, ws_url, mac_address, no_loopback
    );

    Ok((api_url, ws_url, mac_address, no_loopback))
}

/// Initializes the API connection by registering the jam unit and retrying if necessary.
/// 
/// Returns the number of attempts made to establish the connection.
fn init_api_connection(api: &mut dyn JamNationApiTrait) -> Result<usize, BoxError> {
    let mut checks = 1;
    api.jam_unit_register()?;
    debug!("Registered API token");
    while !api.has_token() {
        info!("Can't connect to rtjam-nation. Sleeping 2 seconds then retrying");
        sleep(Duration::new(2, 0));
        checks += 1;
        api.jam_unit_register()?;
    }
    debug!("API connected in {} tries", checks);
    Ok(checks)
}

/// Initializes and spawns a new websocket thread.
/// 
/// This function initializes a new websocket thread using the provided token and URL. It also allows for a custom websocket thread function to be passed in, defaulting to the standard `websocket_thread` function if none is provided.
/// 
/// # Parameters
/// 
/// * `token`: The token to use for authentication in the websocket connection.
/// * `ws_url`: The URL of the websocket endpoint to connect to.
/// * `websocket_thread_fn`: An optional custom function to use for the websocket thread. Defaults to `websocket_thread` if not provided.
/// 
/// # Returns
/// 
/// A tuple containing:
/// 
/// * `mpsc::Sender<WebsockMessage>`: A sender for sending messages to the websocket thread.
/// * `mpsc::Receiver<serde_json::Value>`: A receiver for receiving messages from the websocket thread.
/// * `thread::JoinHandle<()>`: A handle to the spawned websocket thread.
/// 
/// # Errors
/// 
/// This function will return an error if the websocket thread cannot be spawned or if there's an issue with the provided parameters.
fn init_websocket_thread(
    token: &str,
    ws_url: &str,
    websocket_thread_fn: Option<WebSocketThreadFn>,
) -> Result<(mpsc::Sender<WebsockMessage>, mpsc::Receiver<serde_json::Value>, thread::JoinHandle<()>), BoxError> {
    // Use the provided function or default to the websocket_thread
    let websocket_thread_fn = websocket_thread_fn.unwrap_or(websocket_thread);
    debug!("Initializing websocket thread with token: {} and ws_url: {}", token, ws_url);
    
    let (to_ws_tx, to_ws_rx) = mpsc::channel();
    let (from_ws_tx, from_ws_rx) = mpsc::channel();

    let token_clone = token.to_string();
    let ws_url_clone = ws_url.to_string();
    let websocket_handle = thread::spawn(move || {
        if let Err(e) = websocket_thread_fn(&token_clone, &ws_url_clone, from_ws_tx, to_ws_rx) {
            error!("Websocket thread encountered an error: {}", e);
            // Terminate the thread if there's an error
            return()
        }
    });

    // TODO: set up an error channel to relay issues with the websocket thread to the main program
    // For now, there's no easy way to catch errors, thus life can only be good, so let's tell them to be happy
    debug!("Websocket thread spawned successfully");
    Ok((to_ws_tx, from_ws_rx, websocket_handle))
}

fn init_hardware_control() -> Result<(Option<mpsc::Sender<LightMessage>>, Option<thread::JoinHandle<()>>), BoxError> {
    let mut light_option = None;
    let mut hw_handle = None;

    if has_lights() {
        let (lights_tx, lights_rx) = mpsc::channel();
        light_option = Some(lights_tx);

        hw_handle = Some(thread::spawn(move || {
            let _res = hw_control_thread(lights_rx);
        }));
    }

    Ok((light_option, hw_handle))
}

fn start_alsa_thread(engine: JamEngine, in_dev: &str, out_dev: &str) -> Result<thread::JoinHandle<()>, BoxError> {
    let in_dev = in_dev.to_string();
    let out_dev = out_dev.to_string();
    
    let builder = ThreadBuilder::default()
        .name("Real-Time Thread".to_string())
        .priority(ThreadPriority::Max);

    let handle = builder.spawn(move |_result| {
        match alsa_thread::run(engine, &in_dev, &out_dev) {
            Ok(()) => {
                debug!("ALSA ended with OK");
            }
            Err(e) => {
                error!("ALSA exited with error {}", e);
            }
        }
    })?;

    Ok(handle)
}

fn start_jack_thread(engine: JamEngine) -> Result<thread::JoinHandle<()>, BoxError> {
    let handle = thread::spawn(move || {
        match jack_thread::run(engine) {
            Ok(()) => {
                debug!("jack_thread::run OK");
            }
            Err(e) => {
                error!("Jack thread exited with error {}", e);
            }            
        }
    });

    Ok(handle)     
}

fn run_main_loop(
    from_ws_rx: mpsc::Receiver<serde_json::Value>,
    to_ws_tx: mpsc::Sender<WebsockMessage>,
    command_tx: mpsc::Sender<ParamMessage>,
    pedal_tx: mpsc::Sender<PedalBoard>,
    status_data_rx: mpsc::Receiver<serde_json::Value>,
) -> Result<(), BoxError> {
    let mut websock_room_ping = MicroTimer::new(get_micro_time(), 2_000_000);

    loop {
        handle_websocket_messages(&from_ws_rx, &to_ws_tx, &command_tx, &pedal_tx)?;
        handle_status_messages(&status_data_rx, &to_ws_tx)?;
        handle_room_ping(&mut websock_room_ping, &to_ws_tx)?;
        
        sleep(Duration::new(0, 200_000));
    }
}

fn handle_websocket_messages(
    from_ws_rx: &mpsc::Receiver<serde_json::Value>,
    to_ws_tx: &mpsc::Sender<WebsockMessage>,
    command_tx: &mpsc::Sender<ParamMessage>,
    pedal_tx: &mpsc::Sender<PedalBoard>,
) -> Result<(), BoxError> {
    match from_ws_rx.try_recv() {
        Ok(m) => {
            debug!("websocket message: {}", m);
            if let Ok(msg) = ParamMessage::from_json(&m) {
                match msg.param {
                    JamParam::SetAudioInput => {
                        info!("Set audio input: {}", msg);
                        write_string_to_file("soundin.cfg", &msg.svalue);
                    }
                    JamParam::SetAudioOutput => {
                        info!("Set audio output: {}", msg);
                        write_string_to_file("soundout.cfg", &msg.svalue);
                    }
                    JamParam::ListAudioConfig => {
                        let _res = to_ws_tx.send(WebsockMessage::Chat(make_audio_config()));
                    }
                    JamParam::CheckForUpdate => {
                        info!("Check for update requested. Restarting.");
                        std::process::exit(-1);
                    }
                    JamParam::RandomCommand => {
                        info!("Rando: {}", msg);
                        info!("Output: {}", run_a_command(&msg.svalue));
                        let _res = to_ws_tx.send(WebsockMessage::Chat(run_a_command(&msg.svalue)));
                    }
                    JamParam::ShutdownDevice => {
                        info!("Exiting app");
                        std::process::exit(-1);
                    }
                    JamParam::LoadBoard => {
                        let idx = msg.ivalue_1 as usize;
                        if idx < 2 {
                            let mut board = PedalBoard::new(idx);
                            board.load_from_json(&msg.svalue);
                            let _res = pedal_tx.send(board);
                        }
                    }
                    _ => {
                        let _res = command_tx.send(msg);
                    }
                }
            } else {
                warn!("JSON parse Error");
            }
        }
        Err(mpsc::TryRecvError::Empty) => {}
        Err(mpsc::TryRecvError::Disconnected) => warn!("websocket: disconnected channel"),
    }
    Ok(())
}

fn handle_status_messages(
    status_data_rx: &mpsc::Receiver<serde_json::Value>,
    to_ws_tx: &mpsc::Sender<WebsockMessage>,
) -> Result<(), BoxError> {
    match status_data_rx.try_recv() {
        Ok(m) => {
            trace!("audio thread message: {}", m.to_string());
            to_ws_tx.send(WebsockMessage::Chat(m))?;
        }
        Err(mpsc::TryRecvError::Empty) => {}
        Err(mpsc::TryRecvError::Disconnected) => warn!("audio thread: disconnected channel"),
    }
    Ok(())
}

fn handle_room_ping(
    websock_room_ping: &mut MicroTimer,
    to_ws_tx: &mpsc::Sender<WebsockMessage>,
) -> Result<(), BoxError> {
    let now = get_micro_time();
    if websock_room_ping.expired(now) {
        to_ws_tx.send(WebsockMessage::Chat(
            json!({"speaker": "UnitChatRobot", "websockPing": {"isRust": true}}),
        ))?;
        websock_room_ping.reset(now);
    }
    Ok(())
}

fn jam_unit_ping_thread(api: &mut dyn JamNationApiTrait) -> Result<(), BoxError> {
    loop {
        while api.has_token() == true {
            // While in this loop, we are going to ping every 10 seconds
            match api.jam_unit_ping() {
                Ok(ping) => {
                    debug!("jam_unit_ping: {}", ping);
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
                    warn!("jam_unit_ping Error: {}", e);
                }
            }
        }
        if !api.has_token() {
            // We need to register the server
            warn!("jam_unit_ping: no token, registering");
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
            debug!("run command: {}", rval);
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
                    warn!("file not found, creating: {}", fname);
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

#[cfg(test)]
mod init_config {
    use super::*;

    #[test]
    fn test_default() {
        // Test with non-existent file and validate passed in defaults
        // From init_config:
        /*
            let default_params = json::object! {
                "api_url": "http://rtjam-nation.com/api/1/",
                "ws_url": "ws://rtjam-nation.com/primus",
                "no_loopback": false
            };
        */
        let expected_api_url = "http://rtjam-nation.com/api/1/";
        let expected_ws_url = "ws://rtjam-nation.com/primus";
        let expected_no_loopback = false;

        let result = init_config(Some("custom_settings.json"));
        assert!(result.is_ok());
        let (api_url, ws_url, mac_address, no_loopback) = result.unwrap();
        assert_eq!(api_url, expected_api_url);
        assert_eq!(ws_url, expected_ws_url);
        assert!(!mac_address.is_empty());
        assert_eq!(no_loopback, expected_no_loopback);
    }

    #[test]
    fn test_bad_file_name() {
        // Test with custom config file
        let result = init_config(Some("Illegal*File$Name"));
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().starts_with("Invalid filename 'Illegal*File$Name'"));
    }
}

#[cfg(test)]
mod init_websocket_thread {
    use super::*;
    use std::sync::mpsc::{self};

    // Mock function that matches the WebSocketThreadFn signature
    fn mock_websocket_thread(
        token: &str,
        ws_url: &str,
        ws_tx: mpsc::Sender<serde_json::Value>,
        ws_rx: mpsc::Receiver<WebsockMessage>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        assert_eq!(token, "test_token");
        assert_eq!(ws_url, "ws://test.com");
        // Loop to map messages from from_ws_tx to from_ws_rx
        std::thread::spawn(move || {
            for message in ws_rx {
                // Send the received message to the from_ws_tx channel
                let json_message = serde_json::to_value(message).unwrap_or_default();
                let _ = ws_tx.send(json_message);
            }
        });
        
        Ok(())
    }
        
    #[test]
    fn test_websocket_thread_passing_messages() {
        let result = init_websocket_thread("test_token", "ws://test.com", Some(mock_websocket_thread));
        assert!(result.is_ok(), "Error in call to init_websocket_thread()");
        // Use the channels returned from init_websocket_thread
        let (to_ws_tx, from_ws_rx, _) = result.unwrap();

        // Shove a message through and see what comes out the other side
        let payload = json!({"message": "mocked"});
        let message = WebsockMessage::Chat(payload.clone());
        let sent = to_ws_tx.send(message);
        assert!(sent.is_ok(), "Failed to send test message");
        
        // Assert that sending was successful, but give the mock some time do the needful
        thread::sleep(Duration::from_millis(100));
        let received = from_ws_rx.try_recv().unwrap();
        // Extract the "Chat" object from the received message
        if let Some(chat_object) = received.get("Chat") {
            // Now you can compare the extracted chat object with the expected message
            assert_eq!(chat_object, &payload, "Payload doesn't match from Chat message");
        } else {
            assert!(false, "Received message does not contain 'Chat' key");
        }
    }   
}

#[cfg(test)]
mod init_api_connection {
    use super::*;
    //use json::JsonValue;

    struct MockJamNationApi {
        token: Option<String>,
        failure_count: usize,
        register_failure: bool,
    }
    
    // Mock implementation of JamNationApi
    impl MockJamNationApi {
        fn new(failure_count: usize, register_failure: bool) -> Self {
            Self { 
                token: None, 
                failure_count: failure_count,
                register_failure: register_failure,
            }
        }
    }
    
    impl JamNationApiTrait for MockJamNationApi {
        fn jam_unit_register(&mut self) -> Result<(), BoxError> {
            if self.register_failure {
                return Err(BoxError::from("Mock register failure"));
            }

            Ok(())
        }
    
        fn get_token(&self) -> &str {
            self.token.as_ref().map_or("no_token", |token| token.as_str())
        }
    
        fn has_token(&self) -> bool {
            // Declare a static mutable variable to track failures
            static mut CURRENT_FAILURES: usize = 0;
    
            unsafe {
                if CURRENT_FAILURES < self.failure_count {
                    CURRENT_FAILURES += 1; // Increment failure count
                    false // Return false for the specified number of failures
                } else {
                    true // Return true after failures are exhausted
                }
            }
        }

        fn forget_token(&mut self) -> () {
            ()
        }

        fn broadcast_unit_ping(&self) -> Result<serde_json::Value, BoxError> {
            Ok(serde_json::Value::Null)           
        }

        fn jam_unit_ping(&self) -> Result<serde_json::Value, BoxError> {
            Ok(().into())           
        }
    }
    
    #[test]
    fn test_init_api_connection() {
        let mut mock = MockJamNationApi::new(0, false);
        let result = init_api_connection(&mut mock);
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_api_connection_retries() {
        let mut mock = MockJamNationApi::new(3, false); // Set failure count to 3
        let result = init_api_connection(&mut mock);
        // I'm not sure whether to error or not, but for now just verify the number of tries 
        assert!(result.is_ok(), "Expected Ok after 1 initial and 3 retries");
        assert!(result.unwrap() == 4, "Expected total try value of 4");
    }

    #[test]
    fn test_init_api_connection_register_fail() {
        let mut mock = MockJamNationApi::new(1, true); // Set failure count to 1 and register failure flag to true
        let result = init_api_connection(&mut mock);
        assert!(result.is_err(), "Expected Err after 1 initial and 1 retry");
        assert_eq!(result.unwrap_err().to_string(), "Mock register failure");
    }
}

// mod init_hardware_control {
//     // use super::*;
    
//     #[test]
//     fn test_init_hardware_control() {
//         let result = init_hardware_control();
//         assert!(result.is_ok());
        
//         let (light_option, _handle) = result.unwrap();
//         // Light option should be Some if hardware lights are available
//         // None if not available
//         match has_lights() {
//             true => assert!(light_option.is_some()),
//             false => assert!(light_option.is_none())
//         }
//     }