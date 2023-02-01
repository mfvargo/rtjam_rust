//!
//! mixer used to combine all speakers into a stereo channel
//!
//!

use super::channel_strip::ChannelStrip;

pub const MIXER_CHANNELS: usize = 24;

pub struct Mixer {
    master_vol: f32,
    strips: Vec<ChannelStrip>,
}

impl Mixer {
    pub fn build() -> Mixer {
        let mut mixer = Mixer {
            master_vol: 1.0,
            strips: vec![],
        };
        for _ in 0..MIXER_CHANNELS {
            mixer.strips.push(ChannelStrip::new());
        }
        mixer
    }
    pub fn get_master(&self) -> f32 {
        self.master_vol
    }
    pub fn set_master(&mut self, v: f32) -> () {
        self.master_vol = v;
    }
    pub fn get_mix(&mut self, out_a: &mut [f32], out_b: &mut [f32]) -> () {
        // Zero out the out buffer
        for i in 0..out_a.len() {
            out_a[i] = 0.0;
            out_b[i] = 0.0;
        }
        // TODO:  get the mix
        for chan in &mut self.strips {
            chan.mix_into(out_a, out_b);
        }
    }
    pub fn add_to_channel(&mut self, chan_no: usize, audio: &[f32]) -> () {
        if chan_no > MIXER_CHANNELS {
            return;
        }
        self.strips[chan_no].add_data(audio);
    }
}

#[cfg(test)]
mod test_mixer {
    use super::*;

    #[test]
    fn build_mixer() {
        let mut mixer = Mixer::build();
        assert_eq!(mixer.get_master(), 1.0);
        mixer.set_master(0.5);
        assert_eq!(mixer.get_master(), 0.5);
    }
}
