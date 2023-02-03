use std::sync::mpsc;

use serde_json::json;

use crate::{
    common::{box_error::BoxError, jam_packet::JamMessage, stream_time_stat::MicroTimer},
    server::player_list::get_micro_time,
};

use super::{
    channel_map::ChannelMap, jam_socket::JamSocket, mixer::Mixer, param_message::ParamMessage,
};

pub struct JamEngine {
    // gonna have some stuff
    sock: JamSocket,
    recv_message: JamMessage,
    xmit_message: JamMessage,
    status_data_tx: mpsc::Sender<serde_json::Value>,
    command_rx: mpsc::Receiver<ParamMessage>,
    update_timer: MicroTimer,
    token: String,
    mixer: Mixer,
    chan_map: ChannelMap,
    git_hash: String,
}

impl JamEngine {
    pub fn build(
        tx: mpsc::Sender<serde_json::Value>,
        rx: mpsc::Receiver<ParamMessage>,
        tok: &str,
        git_hash: &str,
    ) -> Result<JamEngine, BoxError> {
        let mut engine = JamEngine {
            sock: JamSocket::build(9991)?,
            recv_message: JamMessage::build(),
            xmit_message: JamMessage::build(),
            status_data_tx: tx,
            command_rx: rx,
            update_timer: MicroTimer::build(get_micro_time(), 200_000),
            token: String::from(tok),
            mixer: Mixer::build(),
            chan_map: ChannelMap::new(),
            git_hash: String::from(git_hash),
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
        let now = get_micro_time();
        self.send_status(now);
        self.check_command();
        self.read_network(now);
        self.send_my_audio(in_a, in_b);
        // This is where we would get the playback data
        // For now just copy input to output
        // let (a, b) = self.xmit_message.decode_audio();
        self.mixer.get_mix(out_a, out_b);
        // out_a.clone_from_slice(&a[..]);
        // out_b.clone_from_slice(&b[..]);
        Ok(())
    }
    fn send_status(&mut self, now: u128) -> () {
        // give any clients on the websocket an update
        if self.update_timer.expired(now) {
            self.update_timer.reset(now);
            // send level updates
            let _res = self.status_data_tx.send(self.build_level_event());
            // println!("mixer: {}", self.mixer);
        }
    }
    // This is where we check for any commands we need to process
    fn check_command(&mut self) -> () {
        match self.command_rx.try_recv() {
            Ok(msg) => {
                // received a command
                println!("jack thread received message: {}", msg);
                // TODO: refactor this into some kind of match thing
                if msg.param == 21 {
                    // connect message
                    let _res = self.sock.connect(
                        &msg.svalue,
                        (msg.ivalue_1 as i32).into(),
                        (msg.ivalue_2 as i32).into(),
                    );
                    self.xmit_message.set_client_id(msg.ivalue_2 as u32);
                }
                if msg.param == 22 {
                    self.sock.disconnect();
                    self.xmit_message.set_client_id(0);
                    self.chan_map.clear();
                }
            }
            Err(_) => (),
        }
    }
    // This is where we read packets off of the network
    fn read_network(&mut self, now: u128) -> () {
        self.chan_map.prune(now);
        let mut reading = true;
        while reading {
            let _res = self.sock.recv(&mut self.recv_message);
            match _res {
                Ok(_v) => {
                    // got a network packet
                    // Figure out what channel this guy belongs to
                    let (c1, c2) = self.recv_message.decode_audio();
                    if c1.len() > 0 {
                        // only map and put if it's got some data
                        match self
                            .chan_map
                            .get_loc_channel(self.recv_message.get_client_id(), now)
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
        let data = json!({
            "speaker": "UnitChatRobot",
            "levelEvent": json!({
                  "jamUnitToken": self.token,
                  "connected": self.sock.is_connected(),
                  "git_hash": self.git_hash,
                  "masterLevel": self.mixer.get_master_level_avg(),
                  "peakMaster": self.mixer.get_master_level_peak(),
                  // TODO these values need to take out the gain in the channel strip
                  "inputLeft": self.mixer.get_channel_power_avg(0),
                  "inputRight": self.mixer.get_channel_power_avg(1),
                  "peakLeft": self.mixer.get_channel_power_peak(0),
                  "peakRight": self.mixer.get_channel_power_peak(1),
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
}
