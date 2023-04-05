//! the JamEngine aggregates all the sound components into a single structure.  
//!
//! The engine drives off the [`JamEngine::process`] function
use std::{str::FromStr, sync::mpsc};

use serde_json::json;

use crate::{
    common::{
        box_error::BoxError,
        get_micro_time,
        jam_packet::JamMessage,
        stream_time_stat::{MicroTimer, StreamTimeStat},
    },
    dsp::tuner::Tuner,
    pedals::pedal_board::PedalBoard,
    utils::to_db,
};

use super::{
    channel_map::ChannelMap,
    jam_socket::JamSocket,
    mixer::{Mixer, MIXER_CHANNELS},
    param_message::{JamParam, ParamMessage},
};

// Set a timer for how long a connect will hold up without a keepalive from the web client
pub const IDLE_DISCONNECT: u128 = 15 * 60 * 1000 * 1000; // 15 minutes
pub const IDLE_REFRESH: u128 = 2 * 1000 * 1000; // 2 seconds

/// Aggregates all the sound components into a single structure
///
/// Once built, the audio engine should call the process function every 128 samples
/// to drive the engine.
///
/// The JamEngine maintains:
/// - UDP Socket and Connection state to rooms hosted by broadcast components [`JamSocket`]
/// - Audio Mixer for room members (including the unit itself) [`Mixer`]
/// - ChannelMap to map room members to mixer channels [`ChannelMap`]
/// - PedalBoards for the two local channesl [`PedalBoard`]
/// - Tuners for both incoming channels (to tune your instruments) [`Tuner`]
///
///
/// To avoid having a mutex around the objects in the process loop, the JamEngine is created
/// with a mpsc::Sender and mpsc::Receiver.  The Engine will send json formatted status messages
/// to the Sender and will poll the Receiver every frame for any ParamMessages it should process
/// every process loop.
///
/// # Example
/// ```
/// use std::sync::mpsc;
/// use rtjam_rust::JamEngine;
/// use rtjam_rust::ParamMessage;
/// fn main() {
///     let (status_data_tx, _status_data_rx): (
///             mpsc::Sender<serde_json::Value>,
///             mpsc::Receiver<serde_json::Value>,
///         ) = mpsc::channel();
///     let (_command_tx, command_rx): (
///             mpsc::Sender<ParamMessage>,
///             mpsc::Receiver<ParamMessage>
///         ) = mpsc::channel();
///     let mut engine = JamEngine::new(status_data_tx, command_rx, "someToken", "some_git_hash").expect("oops");
///     // At this point some audio engine would use engine.process() as the callback for audio frames
///     let in_a = [0.0;128];
///     let in_b = [0.0;128];
///     let mut out_left = [0.0;128];
///     let mut out_right = [0.0;128];
///     engine.process(&in_a, &in_b, &mut out_left, &mut out_right);
/// }
///
/// ```
///

pub struct JamEngine {
    // gonna have some stuff
    sock: JamSocket,
    recv_message: JamMessage,
    xmit_message: JamMessage,
    status_data_tx: mpsc::Sender<serde_json::Value>,
    command_rx: mpsc::Receiver<ParamMessage>,
    update_timer: MicroTimer,
    update_fallback_timer: MicroTimer,
    disconnect_timer: MicroTimer,
    debug_timer: MicroTimer,
    token: String,
    mixer: Mixer,
    chan_map: ChannelMap,
    git_hash: String,
    now: u128,
    pedal_boards: Vec<PedalBoard>,
    tuners: [Tuner; 2],
    jack_jitter: StreamTimeStat,
}

impl JamEngine {
    /// create a JamEngine with this call.  The engine requires a mpsc Sender that it will
    /// use to send json formatted status messages that will get routed to the websocket
    /// so the U/X can display the engine and its settings.  It also requires a mpsc Receiver
    /// to get commands to modify it's behavior.  See [`ParamMessage`] for details. It needs the
    /// token to pass through to the U/X so it can be sure it's talking to the right device.
    /// And it needs the git_hash to pass through to the U/X for software update checking.
    ///
    /// See [`crate::sound::client`]
    pub fn new(
        tx: mpsc::Sender<serde_json::Value>,
        rx: mpsc::Receiver<ParamMessage>,
        tok: &str,
        git_hash: &str,
    ) -> Result<JamEngine, BoxError> {
        let now = get_micro_time();
        let mut engine = JamEngine {
            sock: JamSocket::new(9991)?,
            recv_message: JamMessage::new(),
            xmit_message: JamMessage::new(),
            status_data_tx: tx,
            command_rx: rx,
            update_timer: MicroTimer::new(now, IDLE_REFRESH),
            update_fallback_timer: MicroTimer::new(now, IDLE_REFRESH * 5),
            disconnect_timer: MicroTimer::new(now, IDLE_DISCONNECT), // 15 minutes in uSeconds
            debug_timer: MicroTimer::new(now, 500_000),
            token: String::from(tok),
            mixer: Mixer::new(),
            chan_map: ChannelMap::new(),
            git_hash: String::from(git_hash),
            now: now,
            pedal_boards: vec![PedalBoard::new(), PedalBoard::new()],
            tuners: [Tuner::new(), Tuner::new()],
            jack_jitter: StreamTimeStat::new(100),
        };
        // Set out client id to some rando number when not connected
        engine.xmit_message.set_client_id(4321);
        Ok(engine)
    }
    /// This is the function that the audio engine will call with frames of data.  The four arguments are the
    /// two input channels for the component, and the stereo output.
    ///
    /// All control messages should be sent via the mpsc::Receiver passed into new above.
    pub fn process(
        &mut self,
        in_a: &[f32],
        in_b: &[f32],
        out_a: &mut [f32],
        out_b: &mut [f32],
    ) -> Result<(), BoxError> {
        // Get the local microsecond time
        self.set_now();
        self.send_status();
        self.check_disconnect();
        self.check_command();
        self.read_network();
        self.send_my_audio(in_a, in_b);
        // This is where we would get the playback data
        // For now just copy input to output
        // let (a, b) = self.xmit_message.decode_audio();
        self.mixer.get_mix(out_a, out_b);
        // out_a.clone_from_slice(&a[..]);
        // out_b.clone_from_slice(&b[..]);
        self.debug_output();
        Ok(())
    }
    fn debug_output(&mut self) {
        if self.debug_timer.expired(self.now) {
            self.debug_timer.reset(self.now);
            // println!("disconnect: {}", self.disconnect_timer.since(self.now));
            println!(
                "jack_jitter: {:.2}, {:.2}",
                self.jack_jitter.get_mean(),
                self.jack_jitter.get_sigma()
            );
            println!("mixer: {}", self.mixer);
            println!("map: {}", self.chan_map);
        }
    }
    fn set_now(&mut self) -> () {
        let now = get_micro_time();
        self.jack_jitter.add_sample((now - self.now) as f64);
        self.now = now;
    }
    fn check_disconnect(&mut self) -> () {
        if self.disconnect_timer.expired(self.now) {
            self.disconnect();
        }
    }
    fn disconnect(&mut self) -> () {
        self.sock.disconnect();
        // self.xmit_message.set_client_id(0);
        self.chan_map.clear();
    }
    fn connect(&mut self, server: &str, port: i64, id: i64) -> () {
        let _res = self.sock.connect(server, port, id);
        self.xmit_message.set_client_id(id as u32);
        self.disconnect_timer.reset(self.now);
    }
    fn send_status(&mut self) -> () {
        // give any clients on the websocket an update
        if self.update_timer.expired(self.now) {
            self.update_timer.reset(self.now);
            if self.update_fallback_timer.expired(self.now) {
                // throttle back to default refresh interval
                self.update_timer.set_interval(IDLE_REFRESH);
            }
            // send level updates
            let event = self.build_level_event();
            let _res = self.status_data_tx.send(event);
            // println!("disconnect: {}", self.disconnect_timer.since(self.now));
            // println!("mixer: {}", self.mixer);
        }
    }
    // This is where we check for any commands we need to process
    fn check_command(&mut self) -> () {
        match self.command_rx.try_recv() {
            Ok(msg) => {
                self.process_param_command(msg);
            }
            Err(_) => (),
        }
    }
    // This is where we read packets off of the network
    fn read_network(&mut self) -> () {
        self.chan_map.prune(self.now);
        let mut reading = true;
        while reading {
            let _res = self.sock.recv(&mut self.recv_message);
            match _res {
                Ok(_v) => {
                    // got a network packet
                    // Set the server timestamp on xmit packets to loop it back to broadcast server
                    self.xmit_message
                        .set_server_time(self.recv_message.get_server_time());
                    // Figure out what channel this guy belongs to
                    let (c1, c2) = self.recv_message.decode_audio();
                    if c1.len() > 0 {
                        // only map and put if it's got some data
                        match self.chan_map.get_loc_channel(
                            self.recv_message.get_client_id(),
                            self.now,
                            self.recv_message.get_sequence_num(),
                        ) {
                            Some(idx) => {
                                // We found a channel.
                                self.mixer.add_to_channel(idx, &c1);
                                self.mixer.add_to_channel(idx + 1, &c2);
                            }
                            None => {
                                // For some reason we can't get a channel for this packet.
                            }
                        }
                    }
                }
                Err(_e) => {
                    // This is where we get WouldBlock when there is nothing to read
                    reading = false;
                }
            }
        }
    }
    // This is where we forward our data to the network (if connected)
    fn send_my_audio(&mut self, in_a: &[f32], in_b: &[f32]) -> () {
        let mut a_temp: Vec<f32> = vec![0.0; in_a.len()];
        let mut b_temp: Vec<f32> = vec![0.0; in_b.len()];
        self.tuners[0].add_samples(in_a);
        self.tuners[0].get_note();
        self.tuners[1].add_samples(in_b);
        self.tuners[1].get_note();
        self.pedal_boards[0].process(in_a, &mut a_temp);
        self.pedal_boards[1].process(in_b, &mut b_temp);
        self.xmit_message.encode_audio(&a_temp, &b_temp);
        let _res = self.sock.send(&mut self.xmit_message);
        // Stuff my buffers into the mixer for local monitoring
        self.mixer.add_to_channel(0, &a_temp);
        self.mixer.add_to_channel(1, &b_temp);
    }
    fn build_level_event(&mut self) -> serde_json::Value {
        let mut players: Vec<serde_json::Value> = vec![];
        players.push(json!(
            {
                "clientId": self.xmit_message.get_client_id(),
                "depth": self.mixer.get_depth_in_msec(0),  // convert to msec
                "level0": self.mixer.get_channel_power_avg(0),
                "level1": self.mixer.get_channel_power_avg(1),
                "peak0": self.mixer.get_channel_power_peak(0),
                "peak1": self.mixer.get_channel_power_peak(1),
                "drops": 0,
            }
        ));
        let mut idx: usize = 2;
        for c in self.chan_map.get_clients() {
            if !c.is_empty() {
                players.push(json!(
                    {
                        "clientId": c.client_id,
                        "depth": self.mixer.get_depth_in_msec(idx),
                        "level0": self.mixer.get_channel_power_avg(idx),
                        "level1": self.mixer.get_channel_power_avg(idx+1),
                        "peak0": self.mixer.get_channel_power_peak(idx),
                        "peak1": self.mixer.get_channel_power_peak(idx+1),
                        "drops": c.get_drops(),
                    }
                ));
            }
            idx += 2;
        }
        // this is a hack for input gain on channel 0 and 1
        let in_gain0 = to_db(self.mixer.get_channel_gain(0)).round();
        let in_gain1 = to_db(self.mixer.get_channel_gain(1)).round();
        let data = json!({
            "speaker": "UnitChatRobot",
            "levelEvent": json!({
                  "jamUnitToken": self.token,
                  "connected": self.sock.is_connected(),
                  "git_hash": self.git_hash,
                  "masterLevel": self.mixer.get_master_level_avg(),
                  "peakMaster": self.mixer.get_master_level_peak(),
                  // TODO these values need to take out the gain in the channel strip
                  "inputLeft": self.mixer.get_channel_power_avg(0) - in_gain0,
                  "inputRight": self.mixer.get_channel_power_avg(1) - in_gain1,
                  "peakLeft": self.mixer.get_channel_power_peak(0) - in_gain0,
                  "peakRight": self.mixer.get_channel_power_peak(1)- in_gain1,
                  // These are what the channel is sending to the room
                  "roomInputLeft": self.mixer.get_channel_power_avg(0),
                  "roomInputRight": self.mixer.get_channel_power_avg(1),
                  "roomPeakLeft": self.mixer.get_channel_power_peak(0),
                  "roomPeakRight": self.mixer.get_channel_power_peak(1),
                  // TODO  These are stubs for now
                  "inputLeftFreq": self.tuners[0].get_note(),
                  "inputRightFreq": self.tuners[1].get_note(),
                  "leftTunerOn": self.tuners[0].enable,
                  "rightTunerOn": self.tuners[1].enable,
                  "leftRoomMute": false,
                  "rightRoomMute": false,
                  "beat": 0,
                  "jsonTimeStamp": 0,
                  "midiDevice": "not supported",
                  "players": players,
            })
        });

        data
    }
    fn process_param_command(&mut self, msg: ParamMessage) -> () {
        match msg.param {
            JamParam::ChanGain1 => {
                self.mixer.set_channel_gain(0, msg.fvalue);
            }
            JamParam::ChanGain2 => {
                self.mixer.set_channel_gain(1, msg.fvalue);
            }
            JamParam::ChanGain3 => {
                self.mixer.set_channel_gain(2, msg.fvalue);
            }
            JamParam::ChanGain4 => {
                self.mixer.set_channel_gain(3, msg.fvalue);
            }
            JamParam::ChanGain5 => {
                self.mixer.set_channel_gain(4, msg.fvalue);
            }
            JamParam::ChanGain6 => {
                self.mixer.set_channel_gain(5, msg.fvalue);
            }
            JamParam::ChanGain7 => {
                self.mixer.set_channel_gain(6, msg.fvalue);
            }
            JamParam::ChanGain8 => {
                self.mixer.set_channel_gain(7, msg.fvalue);
            }
            JamParam::ChanGain9 => {
                self.mixer.set_channel_gain(8, msg.fvalue);
            }
            JamParam::ChanGain10 => {
                self.mixer.set_channel_gain(9, msg.fvalue);
            }
            JamParam::ChanGain11 => {
                self.mixer.set_channel_gain(10, msg.fvalue);
            }
            JamParam::ChanGain12 => {
                self.mixer.set_channel_gain(11, msg.fvalue);
            }
            JamParam::ChanGain13 => {
                self.mixer.set_channel_gain(12, msg.fvalue);
            }
            JamParam::ChanGain14 => {
                self.mixer.set_channel_gain(13, msg.fvalue);
            }
            JamParam::SetFader => {
                if Self::check_index(msg.ivalue_1 as usize) {
                    self.mixer
                        .set_channel_fade(msg.ivalue_1 as usize, msg.fvalue as f32);
                }
            }
            JamParam::RoomChange => {
                // connect message
                self.connect(&msg.svalue, msg.ivalue_1, msg.ivalue_2);
            }
            JamParam::Disconnect => {
                self.disconnect();
            }
            JamParam::InsertPedal => {
                let idx = msg.ivalue_1 as usize;
                if idx < self.pedal_boards.len() {
                    self.pedal_boards[idx].insert_pedal(&msg.svalue, msg.ivalue_2 as usize)
                }
                self.send_pedal_info();
            }
            JamParam::DeletePedal => {
                let idx = msg.ivalue_1 as usize;
                if idx < self.pedal_boards.len() {
                    self.pedal_boards[idx].delete_pedal(msg.ivalue_2 as usize);
                }
                self.send_pedal_info();
            }
            JamParam::MovePedal => {
                let idx = msg.ivalue_1 as usize;
                let from_idx: usize = msg.ivalue_2 as usize;
                let to_idx: usize = msg.fvalue.round() as usize;
                if idx < self.pedal_boards.len() {
                    self.pedal_boards[idx].move_pedal(from_idx, to_idx);
                }
                self.send_pedal_info();
            }
            JamParam::LoadBoard => {
                let idx = msg.ivalue_1 as usize;
                if idx < self.pedal_boards.len() {
                    self.pedal_boards[idx].load_from_json(&msg.svalue);
                }
                self.send_pedal_info();
            }
            JamParam::TuneChannel => {
                let idx = msg.ivalue_1 as usize;
                if idx < 2 {
                    self.tuners[idx].enable = msg.ivalue_2 == 1;
                }
            }
            JamParam::SetEffectConfig => {
                // Change a parameter on a pedal
                let idx = msg.ivalue_1 as usize;
                if idx < self.pedal_boards.len() {
                    match serde_json::Value::from_str(&msg.svalue) {
                        Ok(setting) => {
                            self.pedal_boards[idx].change_value(msg.ivalue_2 as usize, &setting);
                        }
                        Err(e) => {
                            // error parsing json to modify a setting
                            dbg!(e);
                        }
                    }
                }
            }
            JamParam::ConnectionKeepAlive => {
                // Sent by web client to let us know they are still there.
                self.disconnect_timer.reset(self.now);
            }
            JamParam::SetUpdateInterval => {
                // Update the refresh rate
                let mut interval = (msg.ivalue_1 * 1000) as u128; // convert to msec
                if interval < 150_000 {
                    interval = 150_000;
                }
                if interval > IDLE_REFRESH {
                    interval = IDLE_REFRESH;
                }
                self.update_timer.set_interval(interval);
                self.update_fallback_timer.reset(self.now);
            }
            JamParam::GetConfigJson => {
                self.send_pedal_info();
            }
            JamParam::GetPedalTypes => {
                let _res = self.status_data_tx.send(json!({
                    "speaker": "UnitChatRobot",
                    "pedalTypes": PedalBoard::get_pedal_types()
                }));
            }
            _ => {
                println!("unknown command: {}", msg);
            }
        }
    }
    fn check_index(idx: usize) -> bool {
        idx < MIXER_CHANNELS
    }
    fn send_pedal_info(&self) -> () {
        let _res = self.status_data_tx.send(json!({
            "speaker": "UnitChatRobot",
            "pedalInfo": [
                self.pedal_boards[0].as_json(0),
                self.pedal_boards[1].as_json(1)
            ]
        }));
    }
}

#[cfg(test)]

mod test_jam_engine {
    use super::*;

    fn build_one() -> JamEngine {
        // This is the channel the audio engine will use to send us status data
        let (status_data_tx, _status_data_rx): (
            mpsc::Sender<serde_json::Value>,
            mpsc::Receiver<serde_json::Value>,
        ) = mpsc::channel();

        // This is the channel we will use to send commands to the jack engine
        let (_command_tx, command_rx): (mpsc::Sender<ParamMessage>, mpsc::Receiver<ParamMessage>) =
            mpsc::channel();

        JamEngine::new(status_data_tx, command_rx, "someToken", "some_git_hash").unwrap()
    }

    #[test]
    fn disconnect_timer() {
        // It should have a disconnect timer
        let mut engine = build_one();
        assert_eq!(engine.disconnect_timer.expired(engine.now), false);
        engine.now = engine.now + IDLE_DISCONNECT + 1;
        assert_eq!(engine.disconnect_timer.expired(engine.now), true);
    }
}
