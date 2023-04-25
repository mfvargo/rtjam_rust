use std::{sync::mpsc, thread::sleep, time::Duration};

use crate::common::{box_error::BoxError, jam_packet::{JamMessage}};

use super::cmd_message::RoomCommandMessage;


pub fn run(
    cmd_rx: mpsc::Receiver<RoomCommandMessage>,
    packet_tx: mpsc::Sender<JamMessage>
) -> Result<(), BoxError> {
    println!("playback thread");
    loop {
        match cmd_rx.try_recv() {
            Ok(m) => {
                // Message from control
                dbg!(m);
            }
            Err(_e) => {
                // ignore error for now
            }
        }
        // get playback
        if let Some(packet) = get_playback() {
            packet_tx.send(packet)?;
        }
    }
    // Ok(())
}

fn get_playback() -> Option<JamMessage> {
    // This is the timer between channel polling
    sleep(Duration::new(0, 2_666_666)); // interpacket delay

    // Get a JamPacket
    let mut packet = JamMessage::new();
    packet.set_client_id(40001);  // TODO:  This is some hack for room playback
    packet.set_sequence_num(1); // TODO:  Need this to be monotonically increasing
    // Need to have the mixer and etc.
    let left: [f32; 128] = [0.0; 128];
    let right: [f32; 128] = [0.0; 128];
    packet.encode_audio(&left, &right);
    Some(packet)
    // None
}