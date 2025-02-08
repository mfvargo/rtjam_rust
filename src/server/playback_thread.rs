use std::{sync::mpsc, thread::sleep, time::Duration};

use log::{info, trace, warn};

use crate::{
    common::{box_error::BoxError, get_micro_time, jam_packet::JamMessage, stream_time_stat::MicroTimer, websock_message::WebsockMessage},
    server::{cmd_message::RoomParam, playback_mixer::PlaybackMixer},
};

use super::cmd_message::RoomCommandMessage;

const FRAME_TIME: u128 = 2_667;
/// This thread will pump out playback packets to the audio_thread  (by writing to packet_tx)
/// It will collect recorded packets from a file, push them into a mixer (all flat settings),
/// then pull them out of the Mixer into a new packet that gets pumped to the audio_thread.
pub fn run(
    to_ws_tx: mpsc::Sender<WebsockMessage>,
    cmd_rx: mpsc::Receiver<RoomCommandMessage>,
    packet_tx: mpsc::Sender<JamMessage>
) -> Result<(), BoxError> {
    info!("playback thread");
    let mut mixer = PlaybackMixer::new();
    let mut now = get_micro_time();
    let mut pback_timer = MicroTimer::new(now, FRAME_TIME);
    let mut transport_update_timer = MicroTimer::new(now, 333_000);

    loop {
        let mut nanos = (FRAME_TIME - pback_timer.since(now)) * 1000;
        nanos = nanos.clamp(0,100_000);
        sleep(Duration::new(0, nanos as u32));
        now = get_micro_time();
        while pback_timer.expired(now) {
            pback_timer.advance(FRAME_TIME);
            // Pull a packet out of the mixer and send it
            match mixer.get_a_packet(now) {
                Some(p) => {
                    packet_tx.send(p)?;
                }
                None => {
                    // Nothing to send
                }
            }
        }
        if transport_update_timer.expired(now) {
            transport_update_timer.reset(now);
            // send transport update
            trace!("playback status {}", mixer.get_status());
            to_ws_tx.send(WebsockMessage::Chat(serde_json::json!({
                "speaker": "RoomChatRobot",
                "playbackStatus": mixer.get_status(),
            })))?;
        }
        match mixer.load_up_till_now(now) {
            Ok(()) => {}
            Err(_e) => {
                // dbg!(e);
                // Probably was end of file.  stop playback
                mixer.close_stream();
                // let _err = mixer.seek_to(now, 0);
            }
        }
        // Check for a command before looping again.
        match cmd_rx.try_recv() {
            Ok(m) => {
                // Message from control
                match m.param {
                    RoomParam::Play => {
                        let file = format!("recs/{}", m.svalue);
                        match mixer.open_stream(&file, now, m.ivalue_1.clamp(0, 100) as usize) {
                            Ok(()) => {}
                            Err(e) => { warn!("open error {:?}", e); }
                        }
                    }
                    RoomParam::Stop => {
                        mixer.close_stream();
                    }
                    RoomParam::Seek => {
                        match mixer.seek_to(now, m.ivalue_1 as usize) {
                            Ok(()) => {}
                            Err(e) => { warn!("seek error {:?}", e); }
                        }
                    }
                    _ => {}
                }
                dbg!(m);
            }
            Err(_e) => {
                // ignore error for now
            }
        }
    }
}
