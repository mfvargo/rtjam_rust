use crate::utils::clip_float;

use super::{fader::Fader, jitter_buffer::JitterBuffer};

pub struct ChannelStrip {
    fader: Fader,
    gain: f32,
    buffer: JitterBuffer,
}

impl ChannelStrip {
    pub fn new() -> ChannelStrip {
        ChannelStrip {
            fader: Fader::new(),
            gain: 1.0,
            buffer: JitterBuffer::build(),
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
    pub fn set_fade(&mut self, v: f32) -> () {
        self.fader.set(clip_float(v));
    }
    pub fn mix_into(&mut self, out_a: &mut [f32], out_b: &mut [f32]) -> () {
        // First get some data from the buff
        let samps = self.buffer.get(out_a.len());
        let mut i: usize = 0;
        for v in samps {
            let (l, r) = self.calc_values(v);
            out_a[i] = out_a[i] + l;
            out_b[i] = out_b[i] + r;
            i += 1;
        }
    }
}
