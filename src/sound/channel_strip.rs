//! represents a channel in the mixer.
//!
//! Each strip has a fader and gain control.  It also has a PowerMeter that
//! tells the power level for data running through the strip.
//!
//! the strip also has a JitterBuffer  to which samples are stored.  When
//! the strip is pulled for data, the samples come from the JitterBuffer
//!
//! # Example
//! ```
//! use rtjam_rust::sound::channel_strip::ChannelStrip;
//!
//! fn main() {
//!     // Build a strip
//!     let mut strip = ChannelStrip::new();
//!     // change settings
//!     strip.set_gain(0.5);  // -6dB  note values are linear!
//!     strip.set_fade(-0.3);   // Pan left
//!     let input = [0.0; 128];
//!     // Add some data to it (this would be from the network...)
//!     strip.add_data(&input);
//!     let mut output_left = [0.0; 128];
//!     let mut output_right = [0.0; 128];
//!     // mix output from the channel into the output buffers applying gain and fade
//!     strip.mix_into(&mut output_left, &mut output_right);
//! }
//! ```
use crate::dsp::power_meter::PowerMeter;
use std::fmt;

use super::{fader::Fader, jitter_buffer::JitterBuffer};

/// represents a channel in a mixer
pub struct ChannelStrip {
    fader: Fader,
    gain: f64,
    buffer: JitterBuffer,
    level: PowerMeter,
}

impl ChannelStrip {
    pub fn new() -> ChannelStrip {
        ChannelStrip {
            fader: Fader::new(),
            gain: 1.0,
            buffer: JitterBuffer::new(),
            level: PowerMeter::new(),
        }
    }
    fn calc_values(&self, in_val: f32) -> (f32, f32) {
        (
            self.gain as f32 * in_val * self.fader.left(),
            self.gain as f32 * in_val * self.fader.right(),
        )
    }
    /// set the gain on the strip.  note v is linear and not in dB
    pub fn set_gain(&mut self, v: f64) -> () {
        self.gain = f64::clamp(v, 0.0, 8.0);
    }
    /// retrieve the current gain setting
    pub fn get_gain(&self) -> f64 {
        self.gain
    }
    /// set the fade value on the channels fader.  -1.0 hard left, +1.0 hard right
    pub fn set_fade(&mut self, v: f32) -> () {
        self.fader.set(v);
    }
    /// Call this pull data out of the strip and mix it into the output frames
    ///
    /// This will get data out of the strips JitterBuffer and apply the strip
    /// settings for gain and fade
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
    /// Get the average power from the strips PowerMeter
    pub fn get_power_avg(&self) -> f64 {
        self.level.get_avg()
    }
    /// Get the peak power from the strips PowerMeter
    pub fn get_power_peak(&self) -> f64 {
        self.level.get_peak()
    }
    /// get the strip's jitter buffer's average depth  
    pub fn get_depth(&self) -> f64 {
        self.buffer.avg_depth()
    }

    /// push audio data into the channels jitter buffer
    pub fn add_data(&mut self, audio: &[f32]) -> () {
        self.buffer.append(audio);
    }
}

impl fmt::Display for ChannelStrip {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[ gain: {:.2}, fade: {:.2}, peak: {:.2}, avg: {:.2}, \tbuffer: {} ]",
            self.gain,
            self.fader,
            self.level.get_peak(),
            self.level.get_avg(),
            self.buffer
        )?;
        write!(f, " ]\n")
    }
}
