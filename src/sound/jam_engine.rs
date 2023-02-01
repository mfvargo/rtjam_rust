use std::sync::mpsc;

use serde_json::json;

use crate::{
    common::{box_error::BoxError, jam_packet::JamMessage, stream_time_stat::MicroTimer},
    server::player_list::get_micro_time,
};

use super::{jam_socket::JamSocket, mixer::Mixer, param_message::ParamMessage};

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
}

impl JamEngine {
    pub fn build(
        tx: mpsc::Sender<serde_json::Value>,
        rx: mpsc::Receiver<ParamMessage>,
        tok: &str,
    ) -> Result<JamEngine, BoxError> {
        Ok(JamEngine {
            sock: JamSocket::build(9991)?,
            recv_message: JamMessage::build(),
            xmit_message: JamMessage::build(),
            status_data_tx: tx,
            command_rx: rx,
            update_timer: MicroTimer::build(get_micro_time(), 1_000_000),
            token: String::from(tok),
            mixer: Mixer::build(),
        })
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
    fn send_status(&mut self, now: u128) -> () {
        // give any clients on the websocket an update
        if self.update_timer.expired(now) {
            self.update_timer.reset(now);
            // send level updates
            let _res = self.status_data_tx.send(json!({
                "speaker": "UnitChatRobot",
                "levelEvent": json!({
                    "connected": self.sock.is_connected(),
                    "players": [],
                    "jamUnitToken": self.token,
                    "masterLevel": -20.0,
                    "peakMaster": -20.0,
                    "inputLeft": -20.0,
                    "inputRight": -20.0,
                    "peakLeft": -20.0,
                    "peakRight": -20.0,
                })
            }));
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
                }
            }
            Err(_) => (),
        }
    }
    // This is where we read packets off of the network
    fn read_network(&mut self) -> () {
        let mut reading = true;
        while reading {
            let _res = self.sock.recv(&mut self.recv_message);
            match _res {
                Ok(_v) => {
                    // got a network packet
                    let (_c1, _c2) = self.recv_message.decode_audio();
                    // TODO:  Stuff it into the jitter buffer
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
}
