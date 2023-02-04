use crate::{dsp::power_meter::PowerMeter, utils::clip_float};
use std::fmt;

use super::{fader::Fader, jitter_buffer::JitterBuffer};

pub struct ChannelStrip {
    fader: Fader,
    gain: f32,
    buffer: JitterBuffer,
    level: PowerMeter,
}

impl ChannelStrip {
    pub fn new() -> ChannelStrip {
        ChannelStrip {
            fader: Fader::new(),
            gain: 1.0,
            buffer: JitterBuffer::build(),
            level: PowerMeter::new(),
        }
    }
    pub fn calc_values(&self, in_val: f32) -> (f32, f32) {
        (
            self.gain * in_val * self.fader.left(),
            self.gain * in_val * self.fader.right(),
        )
    }
    pub fn set_gain(&mut self, v: f32) -> () {
        self.gain = clip_float(v);
    }
    pub fn get_gain(&self) -> f32 {
        self.gain
    }
    pub fn set_fade(&mut self, v: f32) -> () {
        self.fader.set(clip_float(v));
    }
    pub fn mix_into(&mut self, out_a: &mut [f32], out_b: &mut [f32]) -> () {
        // First get some data from the buff
        let samps = self.buffer.get(out_a.len());
        self.level.add_frame(&samps, self.gain);
        let mut i: usize = 0;
        for v in samps {
            let (l, r) = self.calc_values(v);
            out_a[i] = out_a[i] + l;
            out_b[i] = out_b[i] + r;
            i += 1;
        }
    }
    pub fn get_power_avg(&self) -> f64 {
        self.level.get_avg()
    }
    pub fn get_power_peak(&self) -> f64 {
        self.level.get_peak()
    }
    pub fn get_depth(&self) -> f64 {
        self.buffer.avg_depth()
    }
    pub fn add_data(&mut self, audio: &[f32]) -> () {
        self.buffer.append(audio);
    }
}

impl fmt::Display for ChannelStrip {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[ gain: {:.2}, fade: {:.2}, peak: {:.2}, avg: {:.2}, \n\tbuffer: {} ]",
            self.gain,
            self.fader,
            self.level.get_peak(),
            self.level.get_avg(),
            self.buffer
        )?;
        write!(f, " ]\n")
    }
}
