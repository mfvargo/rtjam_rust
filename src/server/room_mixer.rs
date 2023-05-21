use crate::{common::
    jam_packet::{JamMessage}, 
    sound::{mixer::Mixer, channel_map::ChannelMap}
};

pub struct RoomMixer {
    mixer: Mixer,
    chan_map: ChannelMap,
    seq: u32,
}

impl RoomMixer {
    pub fn new() -> RoomMixer {
        RoomMixer {
            mixer: Mixer::new(),
            chan_map: ChannelMap::new(),
            seq: 0,
        }
    }
    pub fn add_a_packet(&mut self, now: u128, msg: &JamMessage) -> () {
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

    pub fn get_a_packet(&mut self, now: u128) -> JamMessage {
        // Mix out a packet
        let mut out_a: [f32; 128] = [0.0; 128];
        let mut out_b: [f32; 128] = [0.0; 128];
        self.mixer.get_mix(&mut out_a, &mut out_b);
        let mut packet = JamMessage::new();
        packet.set_client_id(40002);
        packet.set_sequence_num(self.seq);
        self.seq += 1;
        packet.set_server_time(now as u64);
        packet.encode_audio(&out_a, &out_b);
        // println!("mixer: {}", self.mixer);
        packet
    }
}