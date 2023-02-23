use std::sync::mpsc;

use serde_json::json;

use crate::{
    common::{box_error::BoxError, jam_packet::JamMessage, stream_time_stat::MicroTimer},
    server::player_list::get_micro_time,
    utils::to_db,
};

use super::{
    channel_map::ChannelMap,
    jam_params::JamParams,
    jam_socket::JamSocket,
    mixer::{Mixer, MIXER_CHANNELS},
    param_message::ParamMessage,
};

// Set a timer for how long a connect will hold up without a keepalive from the web client
pub const IDLE_DISCONNECT: u128 = 15 * 60 * 1000 * 1000; // 15 minutes
pub const IDLE_REFRESH: u128 = 2 * 1000 * 1000; // 2 seconds

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
    token: String,
    mixer: Mixer,
    chan_map: ChannelMap,
    git_hash: String,
    now: u128,
}

impl JamEngine {
    pub fn build(
        tx: mpsc::Sender<serde_json::Value>,
        rx: mpsc::Receiver<ParamMessage>,
        tok: &str,
        git_hash: &str,
    ) -> Result<JamEngine, BoxError> {
        let now = get_micro_time();
        let mut engine = JamEngine {
            sock: JamSocket::build(9991)?,
            recv_message: JamMessage::build(),
            xmit_message: JamMessage::build(),
            status_data_tx: tx,
            command_rx: rx,
            update_timer: MicroTimer::build(now, IDLE_REFRESH),
            update_fallback_timer: MicroTimer::build(now, IDLE_REFRESH * 5),
            disconnect_timer: MicroTimer::build(now, IDLE_DISCONNECT), // 15 minutes in uSeconds
            token: String::from(tok),
            mixer: Mixer::build(),
            chan_map: ChannelMap::new(),
            git_hash: String::from(git_hash),
            now: now,
        };
        // Set out client id to some rando number when not connected
        engine.xmit_message.set_client_id(4321);
        Ok(engine)
    }
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
        Ok(())
    }
    fn set_now(&mut self) -> () {
        self.now = get_micro_time();
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
            let _res = self.status_data_tx.send(self.build_level_event());
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
                        match self
                            .chan_map
                            .get_loc_channel(self.recv_message.get_client_id(), self.now)
                        {
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
        self.xmit_message.encode_audio(in_a, in_b);
        let _res = self.sock.send(&mut self.xmit_message);
        // Stuff my buffers into the mixer for local monitoring
        self.mixer.add_to_channel(0, in_a);
        self.mixer.add_to_channel(1, in_b);
    }
    fn build_level_event(&self) -> serde_json::Value {
        let mut players: Vec<serde_json::Value> = vec![];
        players.push(json!(
            {
                "clientId": self.xmit_message.get_client_id(),
                "depth": self.mixer.get_depth_in_msec(0),  // convert to msec
                "level0": self.mixer.get_channel_power_avg(0),
                "level1": self.mixer.get_channel_power_avg(1),
                "peak0": self.mixer.get_channel_power_peak(0),
                "peak1": self.mixer.get_channel_power_peak(1),
            }
        ));
        let mut idx: usize = 2;
        for c in self.chan_map.get_clients() {
            players.push(json!(
                {
                    "clientId": c.client_id,
                    "depth": self.mixer.get_depth_in_msec(idx),
                    "level0": self.mixer.get_channel_power_avg(idx),
                    "level1": self.mixer.get_channel_power_avg(idx+1),
                    "peak0": self.mixer.get_channel_power_peak(idx),
                    "peak1": self.mixer.get_channel_power_peak(idx+1),
                }
            ));
            idx += 2;
        }
        // this is a hack for input gain on channel 0 and 1
        let in_gain0 = to_db(self.mixer.get_channel_gain(0)).round() as f64;
        let in_gain1 = to_db(self.mixer.get_channel_gain(1)).round() as f64;
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
                  "inputLeftFreq": 220.0,
                  "inputRightFreq": 222.0,
                  "leftTunerOn": false,
                  "rightTunerOn": false,
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
        let param: Option<JamParams> = num::FromPrimitive::from_i64(msg.param);
        match param {
            Some(JamParams::ChanGain1) => {
                self.mixer.set_channel_gain(0, msg.fvalue);
            }
            Some(JamParams::ChanGain2) => {
                self.mixer.set_channel_gain(1, msg.fvalue);
            }
            Some(JamParams::ChanGain3) => {
                self.mixer.set_channel_gain(2, msg.fvalue);
            }
            Some(JamParams::ChanGain4) => {
                self.mixer.set_channel_gain(3, msg.fvalue);
            }
            Some(JamParams::ChanGain5) => {
                self.mixer.set_channel_gain(4, msg.fvalue);
            }
            Some(JamParams::ChanGain6) => {
                self.mixer.set_channel_gain(5, msg.fvalue);
            }
            Some(JamParams::ChanGain7) => {
                self.mixer.set_channel_gain(6, msg.fvalue);
            }
            Some(JamParams::ChanGain8) => {
                self.mixer.set_channel_gain(7, msg.fvalue);
            }
            Some(JamParams::ChanGain9) => {
                self.mixer.set_channel_gain(8, msg.fvalue);
            }
            Some(JamParams::ChanGain10) => {
                self.mixer.set_channel_gain(9, msg.fvalue);
            }
            Some(JamParams::ChanGain11) => {
                self.mixer.set_channel_gain(10, msg.fvalue);
            }
            Some(JamParams::ChanGain12) => {
                self.mixer.set_channel_gain(11, msg.fvalue);
            }
            Some(JamParams::ChanGain13) => {
                self.mixer.set_channel_gain(12, msg.fvalue);
            }
            Some(JamParams::ChanGain14) => {
                self.mixer.set_channel_gain(13, msg.fvalue);
            }
            Some(JamParams::SetFader) => {
                if Self::check_index(msg.ivalue_1 as usize) {
                    self.mixer
                        .set_channel_fade(msg.ivalue_1 as usize, msg.fvalue as f32);
                }
            }
            Some(JamParams::RoomChange) => {
                // connect message
                self.connect(&msg.svalue, msg.ivalue_1, msg.ivalue_2);
            }
            Some(JamParams::Disconnect) => {
                self.disconnect();
            }
            Some(JamParams::ConnectionKeepAlive) => {
                // Sent by web client to let us know they are still there.
                self.disconnect_timer.reset(self.now);
            }
            Some(JamParams::SetUpdateInterval) => {
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
            Some(_) => {
                println!("unknown command: {}", msg);
            }
            None => (),
        }
    }
    fn check_index(idx: usize) -> bool {
        idx < MIXER_CHANNELS
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

        JamEngine::build(status_data_tx, command_rx, "someToken", "some_git_hash").unwrap()
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
