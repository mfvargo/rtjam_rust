use std::sync::mpsc;

use crate::{
    common::{box_error::BoxError, jam_packet::JamMessage},
    server::player_list::get_micro_time,
};

use super::{jam_socket::JamSocket, param_message::ParamMessage};

pub struct JamEngine {
    // gonna have some stuff
    sock: JamSocket,
    recv_message: JamMessage,
    xmit_message: JamMessage,
    status_data_tx: mpsc::Sender<serde_json::Value>,
    command_rx: mpsc::Receiver<ParamMessage>,
}

impl JamEngine {
    pub fn build(
        tx: mpsc::Sender<serde_json::Value>,
        rx: mpsc::Receiver<ParamMessage>,
    ) -> Result<JamEngine, BoxError> {
        Ok(JamEngine {
            sock: JamSocket::build(9991)?,
            recv_message: JamMessage::build(),
            xmit_message: JamMessage::build(),
            status_data_tx: tx,
            command_rx: rx,
        })
    }
    pub fn process(
        &mut self,
        in_a: &[f32],
        in_b: &[f32],
        out_a: &mut [f32],
        out_b: &mut [f32],
    ) -> Result<(), BoxError> {
        let _now = get_micro_time();
        self.check_command();
        self.read_network();
        self.send_my_audio(in_a, in_b);
        // This is where we would get the playback data
        // For now just copy input to output
        out_a.clone_from_slice(in_a);
        out_b.clone_from_slice(in_b);
        Ok(())
    }
    fn check_command(&mut self) -> () {
        match self.command_rx.try_recv() {
            Ok(msg) => {
                // received a command
                println!("jack thread received message: {}", msg);
                if msg.param == 21 {
                    // connect message
                    let _res = self.sock.connect(
                        &msg.svalue,
                        (msg.ivalue_1 as i32).into(),
                        (msg.ivalue_2 as i32).into(),
                    );
                }
            }
            Err(_) => (),
        }
    }
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
    fn send_my_audio(&mut self, in_a: &[f32], in_b: &[f32]) -> () {
        self.xmit_message.encode_audio(in_a, in_b);
        let _res = self.sock.send(&self.xmit_message);
    }
}
