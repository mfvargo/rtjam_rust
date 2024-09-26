//!
//! mixer used to combine all speakers into a stereo channel
//!
//! The mixer is comprised of MIXER_CHANNELS number of [`crate::sound::channel_strip::ChannelStrip`] strips.  This is
//! set to 24 so this will support 12 people in a single jam room
//!
//! the [`crate::sound::jam_engine::JamEngine`] has a mixer that it uses to mix audio from
//! room members into a stereo feed for the audio output device.
use crate::{dsp::power_meter::PowerMeter, utils::to_lin};

use super::channel_strip::ChannelStrip;
use std::fmt;

pub const MIXER_CHANNELS: usize = 24;

pub struct Mixer {
    master_vol: f64,
    master_level: PowerMeter,
    strips: Vec<ChannelStrip>,
}

impl Mixer {
    /// Build a new mixer.  All the channels have default settings (gain 1.0, fade 0.0)
    pub fn new() -> Mixer {
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
    /// master volume for the overall mix
    pub fn get_master(&self) -> f64 {
        self.master_vol
    }
    /// set master volume for the overall mix
    pub fn set_master(&mut self, v: f64) -> () {
        self.master_vol = v;
    }
    /// retrieve avg power of the total mix
    pub fn get_master_level_avg(&self) -> f64 {
        self.master_level.get_avg()
    }
    /// retrieve peak power of the total mix
    pub fn get_master_level_peak(&self) -> f64 {
        self.master_level.get_peak()
    }
    /// retrieve the avg power for a particular channel
    pub fn get_channel_power_avg(&self, idx: usize) -> f64 {
        let mut pow = self.strips[idx].get_power_avg().round();
        if pow < -60.0 {
            pow = -60.0
        }
        pow
    }
    /// retrieve peak power for a particular channel
    pub fn get_channel_power_peak(&self, idx: usize) -> f64 {
        let mut pow = self.strips[idx].get_power_peak().round();
        if pow < -60.0 {
            pow = -60.0
        }
        pow
    }
    /// get the jitter buffer avg depth for a channel
    pub fn get_depth_in_msec(&self, idx: usize) -> f64 {
        self.strips[idx].get_depth() / 48.0 // Convert to msec
    }
    /// set gain on a particular channel
    pub fn set_channel_gain(&mut self, idx: usize, val: f64) -> () {
        self.strips[idx].set_gain(to_lin(val));
    }
    /// get the gain setting for a particular channel
    pub fn get_channel_gain(&self, idx: usize) -> f64 {
        self.strips[idx].get_gain()
    }
    /// set pan for a specific channel
    pub fn set_channel_fade(&mut self, idx: usize, val: f32) -> () {
        self.strips[idx].set_fade(val);
    }
    /// get a frame of audio from the mixer.  this will
    /// - pull audio from all jitter buffers for all channels
    /// - apply channel strip fade and gain
    /// - apply master gain to final mix
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
        // Apply Master Volume
        for i in 0..out_a.len() {
            out_a[i] = out_a[i] * self.master_vol as f32;
            out_b[i] = out_b[i] * self.master_vol as f32;
        }
        // get the output volume
        self.master_level.add_frame(out_a, self.master_vol);
    }
    /// call this to stuff data into one of the channels jitter buffer
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
            if c.get_depth() > 0.1 {
                write!(f, " {}", c)?;
            }
        }
        write!(f, "\n")
    }
}

#[cfg(test)]
mod test_mixer {
    use super::*;

    #[test]
    fn build_mixer() {
        let mut mixer = Mixer::new();
        assert_eq!(mixer.get_master(), 1.0);
        mixer.set_master(0.5);
        assert_eq!(mixer.get_master(), 0.5);
    }
}
