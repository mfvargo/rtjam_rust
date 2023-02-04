//!
//! mixer used to combine all speakers into a stereo channel
//!
//!
use crate::{dsp::power_meter::PowerMeter, utils::to_lin};

use super::channel_strip::ChannelStrip;
use std::fmt;

pub const MIXER_CHANNELS: usize = 24;

pub struct Mixer {
    master_vol: f32,
    master_level: PowerMeter,
    strips: Vec<ChannelStrip>,
}

impl Mixer {
    pub fn build() -> Mixer {
        let mut mixer = Mixer {
            master_vol: 1.0,
            strips: vec![],
            master_level: PowerMeter::new(),
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
    pub fn get_master_level_avg(&self) -> f64 {
        self.master_level.get_avg()
    }
    pub fn get_master_level_peak(&self) -> f64 {
        self.master_level.get_peak()
    }
    pub fn get_channel_power_avg(&self, idx: usize) -> f64 {
        let mut pow = self.strips[idx].get_power_avg().round();
        if pow < -60.0 {
            pow = -60.0
        }
        pow
    }
    pub fn get_channel_power_peak(&self, idx: usize) -> f64 {
        let mut pow = self.strips[idx].get_power_peak().round();
        if pow < -60.0 {
            pow = -60.0
        }
        pow
    }
    pub fn get_depth_in_msec(&self, idx: usize) -> f64 {
        self.strips[idx].get_depth() / 48.0 // Convert to msec
    }
    pub fn set_channel_gain(&mut self, idx: usize, val: f32) -> () {
        self.strips[idx].set_gain(to_lin(val));
    }
    pub fn get_channel_gain(&self, idx: usize) -> f32 {
        self.strips[idx].get_gain()
    }
    pub fn set_channel_fade(&mut self, idx: usize, val: f32) -> () {
        self.strips[idx].set_fade(val);
    }
    pub fn get_mix(&mut self, out_a: &mut [f32], out_b: &mut [f32]) -> () {
        // Zero out the out buffer
        for i in 0..out_a.len() {
            out_a[i] = 0.0;
            out_b[i] = 0.0;
        }
        // get the mix
        for chan in &mut self.strips {
            chan.mix_into(out_a, out_b);
        }
        // get the output volume
        self.master_level.add_frame(out_a, self.master_vol);
    }
    pub fn add_to_channel(&mut self, chan_no: usize, audio: &[f32]) -> () {
        if chan_no > MIXER_CHANNELS {
            return;
        }
        self.strips[chan_no].add_data(audio);
    }
}

impl fmt::Display for Mixer {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n")?;
        for c in &self.strips {
            write!(f, " {}", c)?;
        }
        write!(f, "\n")
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
