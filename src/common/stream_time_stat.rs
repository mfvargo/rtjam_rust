//! used to collect time statistics and time with things should happen.
//!
//! The [`JitterBuffer`](crate::sound::jitter_buffer::JitterBuffer) uses StreamTimeStat
//! to get mean and sigma values on the buffer depth to adapt
//!
//! The MicroTimer is used to trigger periodic events (when to send latency updates)
//! by the broadcast component, or when to update u/x elements in the sound component
use std::f64;
use std::fmt;

use serde::Deserialize;
use serde::Serialize;

use crate::dsp::moving_avg::MovingAverage;

/// moving average filter that collect peak, mean, and sigma values for sequences
#[derive(Debug, Deserialize, Serialize)]
pub struct StreamTimeStat {
    window: u64,
    avg: MovingAverage,
    dev: MovingAverage,
}

impl StreamTimeStat {
    /// create a new stat collector with a specific window size
    pub fn new(window_size: u64) -> StreamTimeStat {
        StreamTimeStat {
            window: window_size,
            avg: MovingAverage::new(window_size as usize),
            dev: MovingAverage::new(window_size as usize),
        }
    }
    pub fn clear(&mut self) -> () {
        self.avg = MovingAverage::new(self.window as usize);
        self.dev = MovingAverage::new(self.window as usize);
    }
    pub fn get_mean(&self) -> f64 {
        self.avg.get_mean()
    }
    pub fn get_sigma(&self) -> f64 {
        f64::sqrt(self.dev.get_total()) / self.dev.get_window() as f64
    }
    pub fn get_window(&self) -> u64 {
        self.window
    }
    /// add a sample to the moving average sequence
    ///
    pub fn add_sample(&mut self, sample: f64) -> () {
        self.avg.add_sample(sample);
        let delta = sample - self.get_mean();
        self.dev.add_sample(delta * delta);
    }
}

impl fmt::Display for StreamTimeStat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ mean: {}, sigma: {} window: {} }}",
            self.get_mean(), self.get_sigma(), self.get_window()
        )
    }
}

#[cfg(test)]
mod test_stream_time_stat {
    use super::*;

    #[test]
    fn build() {
        let stat = StreamTimeStat::new(100);
        assert_eq!(stat.get_mean(), 0.0);
    }
    #[test]
    fn add_sample() {
        let mut stat = StreamTimeStat::new(2);
        stat.add_sample(1.0);
        assert_eq!(stat.get_mean(), 0.5);
        stat.add_sample(1.0);
        stat.add_sample(1.0);
        println!("v: {}", stat);
        assert!(stat.get_mean() > 0.999);
        assert!(stat.get_sigma() < 0.01);
    }
}

/// Timer with microsecond accuracy to let things know when a certain time (or more) passed
#[derive(Debug)]
pub struct MicroTimer {
    last_time: u128,
    interval: u128,
}

impl MicroTimer {
    /// create a new timer with the current microsecond value and the interval (in microseconds)
    pub fn new(now: u128, interval: u128) -> MicroTimer {
        MicroTimer {
            last_time: now,
            interval: interval,
        }
    }
    /// recofigure the interval
    pub fn set_interval(&mut self, interval: u128) -> () {
        self.interval = interval;
    }
    /// check if the timer is expired
    pub fn expired(&self, now: u128) -> bool {
        (self.last_time + self.interval) < now
    }
    /// reset the timer to the value of now
    pub fn reset(&mut self, now: u128) {
        self.last_time = now;
    }
    /// Add to the last time to move timer ahead
    pub fn advance(&mut self, delta: u128) {
        self.last_time += delta;
    }
    /// Ask how long since the last time you were reset
    pub fn since(&mut self, now: u128) -> u128 {
        now - self.last_time
    }
}

#[cfg(test)]
mod test_micro_timer {
    use super::*;

    #[test]
    fn test_expiration() {
        let mut now = 1000;
        let mut mt = MicroTimer::new(now, 100);
        assert!(!mt.expired(now));
        now += 99;
        assert!(!mt.expired(now));
        now += 2;
        assert!(mt.expired(now));
        mt.reset(now);
        assert!(!mt.expired(now));
        assert_eq!(mt.since(now + 10), 10);
        mt.set_interval(9);
        now += 10;
        assert!(mt.expired(now));
    }
}
