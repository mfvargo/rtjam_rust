//! adaptive buffer used to smooth incoming network audio data
//!
//! Allow for gap free playback with the minimum amount of delay.
//!
//! Nature of the buffer is that the writes to it have varying degrees of jitter.  Reads
//! from the buffer are clocked from the underlying audio engine and therefore have very
//! low jitter.  Jittery input, smooth output.
//!
//! Adaptation is based on measuring the deviation in the buffers depth.  This
//! relates directly to the inter-arrival variance of network packets.  A large variance
//! requires a deeper buffer to prevent starves on reads.
//!
//! Another adaptation is that when the buffer overflows (large write delay followed by
//! burst of write calls), the next read call will drain the excess data off the front
//! of the buffer (oldest data gets thrown out) and then return data.  This prevents the
//! buffer depth from driving to the largest inter packet delay.  Net effect is this
//! allows for some gaps in playback in order to drive buffer latency down.
use rand::random;

use crate::{
    common::stream_time_stat::StreamTimeStat,
    dsp::{attack_hold_release::AttackHoldRelease, smoothing_filter::SmoothingFilter},
    utils::to_lin,
};
use std::fmt;

const MIN_DEPTH: usize = 128 * 3;
const MIN_HIGH_WATER: usize = MIN_DEPTH * 2;
const MAX_DEPTH: usize = 8192;
// const MIN_SIGMA: f64 = 5.0;

/// Adaptive buffer for smoothing network audio data
///
/// Note that all adaptation functions are performed on buffer read.  
pub struct JitterBuffer {
    buffer: Vec<f32>,
    depth_stats: StreamTimeStat,
    low_water: usize,
    high_water: usize,
    filling: bool,
    underruns: usize,
    overruns: usize,
    depth_filter: AttackHoldRelease<f64>,
    high_water_filter: SmoothingFilter,
    puts: usize,
    gets: usize,
}

impl fmt::Display for JitterBuffer {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ low: {}, high: {}, underruns: {}, overruns: {}, depth: {:.2}, sigma: {:.2}, filt: {:.2} }}",
            self.low_water,
            self.high_water,
            self.underruns,
            self.overruns,
            self.depth_stats.get_mean(),
            self.depth_stats.get_sigma(),
            self.depth_filter.last_output,
        )
    }
}

impl JitterBuffer {
    /// Build a new default jitterbuffer.  It will adapt from here
    pub fn new() -> JitterBuffer {
        JitterBuffer {
            buffer: Vec::<f32>::new(),
            depth_stats: StreamTimeStat::new(500),
            low_water: MIN_DEPTH,
            high_water: MAX_DEPTH,
            filling: true,
            underruns: 0,
            overruns: 0,
            depth_filter: AttackHoldRelease::new(0.4, 1.0, 2.0, 48000.0 / 128.0),
            high_water_filter: SmoothingFilter::build(2.0, 48_000.0 / 128.0),
            puts: 0,
            gets: 0,
        }
    }
    /// retrieves the current number of samples in the buffer  (just for testing)
    pub fn length(&self) -> usize {
        self.buffer.len()
    }
    /// gets the mean depth for the buffer
    pub fn avg_depth(&self) -> f64 {
        self.depth_stats.get_mean()
    }
    /// How many times has the buffer overflowed (no room at the inn)
    pub fn get_overruns(&self) -> usize {
        self.overruns
    }
    /// How many times has the buffer starved (no water in the bottle)
    pub fn get_underruns(&self) -> usize {
        self.underruns
    }
    /// is the buffer filling to it's target depth
    pub fn is_filling(&self) -> bool {
        self.filling
    }
    /// append data to the buffer
    pub fn append(&mut self, audio: &[f32]) -> () {
        self.puts += 1;
        self.buffer.extend_from_slice(audio);
    }
    /// get will retrieve data from the jitter buffer.  It will always give you a full vector but
    /// it might have zeros if there is no data or the buffer is still filling
    pub fn get(&mut self, count: usize, power: f64) -> Vec<f32> {
        // It should get some data off the buffer
        self.gets += 1;

        // Update the depth stats
        self.depth_stats.add_sample(self.buffer.len() as f64); // Gather depth stats

        // Adjust low water depth based on near or current starve
        self.low_water =
            MIN_DEPTH + (self.depth_filter.get(self.buffer.len() < 256) * 1024.0) as usize;
        // Adjust high-water based on jitter sigma
        self.high_water = self
            .high_water_filter
            .get(MIN_HIGH_WATER as f64 + (self.depth_stats.get_sigma() * 8.0))
            as usize;

        // check if we are done filling
        if self.filling {
            if self.buffer.len() >= self.low_water {
                self.filling = false;
            }
        }

        // First case, we are filling so don't give them anything
        if self.filling {
            // just give comfort noise
            let mut noise = vec![0.0; count];
            let gain = to_lin(power) as f32;
            for i in 0..count {
                noise[i] = (random::<f32>() - 0.5) * 2.0 * gain;
            }
            return noise;
        }

        // Second case see if we have too much data and need to throw some out
        if self.buffer.len() > self.high_water {
            self.overruns += 1;
            self.buffer.drain(..self.buffer.len() - 64);
        }

        // Third case, we have enough data to satisfy
        if self.buffer.len() >= count {
            return self.buffer.drain(..count).collect();
        }

        // This is the onset of an underrun
        self.underruns += 1;
        self.filling = true;

        // The buffer is empty
        if self.buffer.len() == 0 {
            // No data in the buffer
            return vec![0.0; count];
        }

        // consuming the last bits of a partial read
        let remainder = count - self.buffer.len();
        // get the partial data
        let mut rval: Vec<f32> = self.buffer.drain(..).collect();
        // fill zeros on the end
        rval.append(&mut vec![0.0; remainder]);
        return rval;
    }
}

#[cfg(test)]
mod test_jitter_buffer {
    use super::*;

    #[test]
    fn build() {
        // you should be able to build a jitter buffer
        let buf = JitterBuffer::new();
        assert_eq!(buf.length(), 0);
    }
    #[test]
    fn avg_depth() {
        // It should tell you it's avg depth
        let buf = JitterBuffer::new();
        assert_eq!(buf.avg_depth(), 0.0);
    }
    #[test]
    fn put() {
        // It should have an append
        let mut buf = JitterBuffer::new();
        let samples: Vec<f32> = vec![0.2, 0.3, 0.4];
        buf.append(&samples);
        assert_eq!(buf.length(), 3);
    }
    #[test]
    fn get_normal() {
        // It should have a get function
        let mut buf = JitterBuffer::new();
        let samples: Vec<f32> = vec![0.2; MIN_DEPTH];
        assert!(buf.is_filling());
        buf.append(&samples);
        assert_eq!(buf.length(), MIN_DEPTH);
        let res = buf.get(2, -60.0);
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn get_from_empty() {
        let mut buf = JitterBuffer::new();
        let res = buf.get(4, -60.0);
        assert_eq!(res.len(), 4);
        // assert_eq!(res, vec![0.0; 4]);
        assert!(buf.is_filling());
    }
    // #[test]
    // fn overrun_measure() {
    //     let mut buf = JitterBuffer::new();
    //     let samps = vec![0.1; MIN_DEPTH + 10];
    //     buf.append(&samps);
    //     let samps = vec![0.1; MIN_DEPTH + 10];
    //     buf.append(&samps);
    //     buf.get(2);
    //     println!("jitterbuf: {}", buf);
    //     assert!(!buf.is_filling());
    //     assert!(buf.get_overruns() > 0);
    // }
}
