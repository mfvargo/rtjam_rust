use crate::{common::stream_time_stat::StreamTimeStat, dsp::peak_detector::PeakDetector};
use std::fmt;

const MIN_DEPTH: usize = 512;
const MAX_DEPTH: usize = 8192;
const MIN_SIGMA: f64 = 7.0;

pub struct JitterBuffer {
    buffer: Vec<f32>,
    depth_stats: StreamTimeStat,
    target_depth: usize,
    filling: bool,
    underruns: usize,
    overruns: usize,
    depth_filter: PeakDetector,
    puts: usize,
    gets: usize,
}

impl fmt::Display for JitterBuffer {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ target: {}, underruns: {}, overruns: {}, depth: {:.2}, puts: {} }}",
            self.target_depth,
            self.underruns,
            self.overruns,
            self.depth_stats.get_mean(),
            self.puts
        )
    }
}

impl JitterBuffer {
    pub fn build() -> JitterBuffer {
        JitterBuffer {
            buffer: Vec::<f32>::new(),
            depth_stats: StreamTimeStat::build(50),
            target_depth: MIN_DEPTH,
            filling: true,
            underruns: 0,
            overruns: 0,
            depth_filter: PeakDetector::build(0.1, 2.5, 48000 / 128),
            puts: 0,
            gets: 0,
        }
    }
    pub fn length(&self) -> usize {
        self.buffer.len()
    }
    pub fn avg_depth(&self) -> f64 {
        self.depth_stats.get_mean()
    }
    pub fn get_overruns(&self) -> usize {
        self.overruns
    }
    pub fn get_underruns(&self) -> usize {
        self.underruns
    }
    pub fn is_filling(&self) -> bool {
        self.filling
    }
    pub fn append(&mut self, audio: &[f32]) -> () {
        self.puts += 1;
        self.buffer.extend_from_slice(audio);
    }
    // get will retrieve data from the jitter buffer.  It will always give you a full vector but
    // it might have zeros if there is no data.
    pub fn get(&mut self, count: usize) -> Vec<f32> {
        // It should get some data off the buffer
        self.gets += 1;
        self.depth_stats.add_sample(self.buffer.len() as f64); // Gather depth stats

        // Adjust target depth based on jitter sigma
        self.target_depth = MIN_DEPTH
            + self
                .depth_filter
                .get(self.depth_stats.get_sigma() * MIN_SIGMA) as usize;
        if self.target_depth > MAX_DEPTH {
            self.target_depth = MAX_DEPTH;
        }

        // check if we are filling
        if self.filling {
            if self.buffer.len() >= self.target_depth {
                self.filling = false;
            }
        }

        // First case, we are filling so don't give them anything
        if self.filling {
            // just give zeros until we have something
            return vec![0.0; count];
        }

        // Second case see if we have too much data and need to throw some out
        // TODO:  code this to be adaptive
        if self.buffer.len() > self.target_depth {
            self.overruns += 1;
            self.buffer.drain(..self.buffer.len() - self.target_depth);
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
        let buf = JitterBuffer::build();
        assert_eq!(buf.length(), 0);
    }
    #[test]
    fn avg_depth() {
        // It should tell you it's avg depth
        let buf = JitterBuffer::build();
        assert_eq!(buf.avg_depth(), 0.0);
    }
    #[test]
    fn put() {
        // It should have an append
        let mut buf = JitterBuffer::build();
        let samples: Vec<f32> = vec![0.2, 0.3, 0.4];
        buf.append(&samples);
        assert_eq!(buf.length(), 3);
    }
    #[test]
    fn get_normal() {
        // It should have a get function
        let mut buf = JitterBuffer::build();
        let samples: Vec<f32> = vec![0.2; MIN_DEPTH];
        assert!(buf.is_filling());
        buf.append(&samples);
        assert_eq!(buf.length(), MIN_DEPTH);
        let res = buf.get(2);
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn get_from_empty() {
        let mut buf = JitterBuffer::build();
        let res = buf.get(4);
        assert_eq!(res.len(), 4);
        assert_eq!(res, vec![0.0; 4]);
        assert!(buf.is_filling());
    }
    #[test]
    fn overrun_measure() {
        let mut buf = JitterBuffer::build();
        let samps = vec![0.1; MIN_DEPTH + 10];
        buf.append(&samps);
        let samps = vec![0.1; MIN_DEPTH + 10];
        buf.append(&samps);
        buf.get(2);
        println!("jitterbuf: {}", buf);
        assert!(!buf.is_filling());
        assert!(buf.get_overruns() > 0);
    }
}
