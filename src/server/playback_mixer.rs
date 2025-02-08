use std::collections::HashSet;

use serde_json::Value;

use crate::{
    common::{box_error::BoxError, jam_packet::JamMessage, packet_stream::PacketReader},
    sound::{channel_map::ChannelMap, mixer::Mixer},
};

const FRAME_TIME: u128 = 2_667;

pub struct PlaybackMixer {
    mixer: Mixer,
    chan_map: ChannelMap,
    stream: Option<PacketReader>,
    seq: u32,
}

impl PlaybackMixer {
    pub fn new() -> PlaybackMixer {
        let mut mixer = PlaybackMixer {
            mixer: Mixer::new(),
            chan_map: ChannelMap::new(),
            stream: None,
            seq: 0,
        };
        mixer.chan_map.clear();
        mixer.mixer.set_master(-9.0);
        mixer
    }
    pub fn get_status(&self) -> Value {
        if let Some(ref stream) = self.stream {
            return stream.get_status();
        }
        serde_json::json!({
            "state": "idle",
            "offset": 0,
            "file": {
                "name": "",
                "date": "",
                "size": 0,
            },
        })
    }
    pub fn get_ids(&mut self, now: u128) -> Result<HashSet<u32>, BoxError> {
        let mut ids: HashSet<u32> = HashSet::new();
        if let Some(reader) = &mut self.stream {
            let mut looping = true;
            while looping {
                match reader.read_packet() {
                    Ok(()) => {
                        ids.insert(reader.get_packet().get_client_id());
                    }
                    Err(_e) => {
                        looping = false;
                    }
                }
            }
            reader.seek_to(now, 0)?;
        }
        Ok(ids)
    }

    pub fn get_a_packet(&mut self, now: u128) -> Option<JamMessage> {
        if self.stream.is_some() {
            // We are currently playing back.  Mix out a packet
            let mut out_a: [f32; 128] = [0.0; 128];
            let mut out_b: [f32; 128] = [0.0; 128];
            self.mixer.get_mix(0, &mut out_a, &mut out_b);
            let mut packet = JamMessage::new();
            packet.set_client_id(40001);
            packet.set_sequence_num(self.seq);
            self.seq += 1;
            packet.set_server_time(now as u64);
            packet.encode_audio(&out_a, &out_b);
            return Some(packet);
        }
        None
        // if let Some(reader) = &mut self.stream {
        //     match reader.read_up_to(now)
        // }
    }

    pub fn get_mix_channels(&mut self, num_chan: usize) -> Option<Vec<[f32; 128]>> {
        if self.stream.is_some() {
            let mut mix: Vec<[f32; 128]> = Vec::new();
            for i in 0..num_chan {
                let mut out_a: [f32; 128] = [0.0; 128];
                let mut out_b: [f32; 128] = [0.0; 128];
                // Note that the channel is i+2 cause the channel map always reserves
                // the first two channels for the local user (but in this case there is no local user)
                self.mixer.get_chan_mix(i+2, &mut out_a, &mut out_b);
                mix.push(out_a);
            }
            return Some(mix);
        }
        None
    }
    pub fn get_a_frame(&mut self)-> Option<[[f32; 128]; 2]> {
        if self.stream.is_some() {
            // We are currently playing back.  Mix out a packet
            let mut out_a: [f32; 128] = [0.0; 128];
            let mut out_b: [f32; 128] = [0.0; 128];
            self.mixer.get_mix(0, &mut out_a, &mut out_b);
            return Some([out_a, out_b]);
        }
        None
        // if let Some(reader) = &mut self.stream {
        //     match reader.read_up_to(now)
        // }
    }

    pub fn open_stream(&mut self, file_name: &str, now: u128, loc: usize) -> Result<(), BoxError> {
        self.chan_map.clear();
        // TODO:  Flush out any data in the mixer  (channels to jitterbuffer) self.mixer.clear();
        self.seq = 0;
        self.stream = Some(PacketReader::new(file_name, now)?);
        self.seek_to(now, loc)?;
        Ok(())
    }
    pub fn close_stream(&mut self) {
        self.stream = None;
    }
    pub fn seek_to(&mut self, now: u128, loc: usize) -> Result<(), BoxError> {
        match self.stream {
            Some(ref mut r) => {
                r.seek_to(now, loc)?;
            }
            None => {}
        }
        Ok(())
    }

    pub fn micros_till_packet(&self, now: u128) -> u128 {
        if let Some(reader) = &self.stream {
            reader.micros_till_packet(now)
        } else {
            FRAME_TIME
        }
    }

    /// This will load data from the stream into the mixer up to now in time
    pub fn load_up_till_now(&mut self, now: u128) -> Result<(), BoxError> {
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

