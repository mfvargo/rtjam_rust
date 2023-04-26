use std::{sync::mpsc, thread::sleep, time::Duration};

use crate::{common::{box_error::BoxError, jam_packet::{JamMessage}, packet_stream::PacketReader, get_micro_time, stream_time_stat::MicroTimer}, sound::{mixer::Mixer, channel_map::ChannelMap}, server::cmd_message::RoomParam};

use super::cmd_message::RoomCommandMessage;

const FRAME_TIME: u128 = 2_650;
/// This thread will pump out playback packets to the audio_thread  (by writing to packet_tx)
/// It will collect recorded packets from a file, push them into a mixer (all flat settings),
/// then pull them out of the Mixer into a new packet that gets pumped to the audio_thread.
pub fn run(
    cmd_rx: mpsc::Receiver<RoomCommandMessage>,
    packet_tx: mpsc::Sender<JamMessage>
) -> Result<(), BoxError> {
    println!("playback thread");
    let mut mixer = PlaybackMixer::new();
    let mut now = get_micro_time();
    let mut pback_timer = MicroTimer::new(now, FRAME_TIME);
    loop {
        let mut nanos = (2667 - pback_timer.since(now)) * 1000;
        nanos = nanos.clamp(0, 300_000);
        sleep(Duration::new(0, nanos as u32)); // poll every 100 microseconds
        now = get_micro_time();
        match mixer.load_up_till_now(now) {
            Ok(()) => {}
            Err(e) => {
                dbg!(e);
                // Probably was end of file.  stop playback
                mixer.close_stream();
            }
        }
        if pback_timer.expired(now) {
            pback_timer.reset(now);
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
        // Check for a command before looping again.
        match cmd_rx.try_recv() {
            Ok(m) => {
                // Message from control
                match m.param {
                    RoomParam::Play => {
                        let file = format!("recs/{}", m.svalue);
                        match mixer.open_stream(&file, now) {
                            Ok(()) => {}
                            Err(e) => {dbg!(e);}
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

pub struct PlaybackMixer {
    mixer: Mixer,
    chan_map: ChannelMap,
    stream: Option<PacketReader>,
    seq: u32,
}

impl PlaybackMixer {
    pub fn new() -> PlaybackMixer {
        PlaybackMixer {
            mixer: Mixer::new(),
            chan_map: ChannelMap::new(),
            stream: None,
            seq: 0,
        }
    }

    pub fn get_a_packet(&mut self, now: u128) -> Option<JamMessage> {
        if self.stream.is_some() {
            // We are currently playing back.  Mix out a packet
            let mut out_a: [f32; 128] = [0.0; 128];
            let mut out_b: [f32; 128] = [0.0; 128];
            self.mixer.get_mix(&mut out_a, &mut out_b);
            let mut packet = JamMessage::new();
            packet.set_client_id(40001);
            packet.set_sequence_num(self.seq);
            self.seq += 1;
            packet.set_server_time(now as u64);
            packet.encode_audio(&out_a, &out_b);
            // println!("mixer: {}", self.mixer);
            return Some(packet);
        }
        None
        // if let Some(reader) = &mut self.stream {
        //     match reader.read_up_to(now)
        // }
    }

    pub fn open_stream(&mut self, file_name: &str, now: u128) -> Result<(), BoxError> {
        self.chan_map.clear();
        // TODO:  Flush out any data in the mixer  (channels to jitterbuffer) self.mixer.clear();
        self.seq = 0;
        self.stream = Some(PacketReader::new(file_name, now)?);
        Ok(())
    }
    pub fn close_stream(&mut self) {
        self.stream = None;
    }

    /// This will load data from the stream into the mixer up to now in time
    fn load_up_till_now(&mut self, now: u128) -> Result<(), BoxError> {
        if let Some(reader) = &mut self.stream {
            let mut looping = true;
            while looping {
                match reader.read_up_to(now)? {
                    Some(msg) => {
                        // Stuff message into the mixer
                        let (c1, c2) = msg.decode_audio();
                        if c1.len() > 0 {
                            // only map and put if it's got some data
                            match self.chan_map.get_loc_channel(
                                msg.get_client_id(),
                                now,
                                msg.get_sequence_num(),
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
                    None => {
                        looping = false;
                    }
                }
            }
        }
        Ok(())
    }
}

