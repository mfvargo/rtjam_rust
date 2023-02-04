// void StreamTimeStats::clear()
// {
//     peak = 0.0;
//     mean = 0.0;
//     sigma = 0.0;
//     windowSize = 100.0;
// }

use std::f64;
use std::fmt;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub struct StreamTimeStat {
    peak: f64,
    mean: f64,
    sigma: f64,
    window: u64,
}

impl StreamTimeStat {
    pub fn build(window_size: u64) -> StreamTimeStat {
        StreamTimeStat {
            peak: 0.0,
            mean: 0.0,
            sigma: 0.0,
            window: window_size,
        }
    }
    pub fn clear(&mut self) -> () {
        self.peak = 0.0;
        self.mean = 0.0;
        self.sigma = 0.0;
    }
    pub fn get_peak(&self) -> f64 {
        self.peak
    }
    pub fn get_mean(&self) -> f64 {
        self.mean
    }
    pub fn get_sigma(&self) -> f64 {
        self.sigma
    }
    pub fn get_window(&self) -> u64 {
        self.window
    }

    pub fn add_sample(&mut self, sample: f64) -> () {
        if sample > self.peak {
            self.peak = sample;
        } else {
            self.peak = self.peak - 0.05; // TODO: This needs to get generalized
        }
        let scale: f64 = (self.window as f64 - 1.0) / self.window as f64;
        self.mean = scale * (self.mean + sample / self.window as f64);
        self.sigma = scale * (self.sigma + (self.mean - sample).abs() / self.window as f64);
    }
}

impl fmt::Display for StreamTimeStat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ peak: {}, mean: {}, sigma: {} window: {} }}",
            self.peak, self.mean, self.sigma, self.window
        )
    }
}

#[cfg(test)]
mod test_stream_time_stat {
    use super::*;

    #[test]
    fn build() {
        let stat = StreamTimeStat::build(100);
        assert_eq!(stat.get_mean(), 0.0);
    }
    #[test]
    fn add_sample() {
        let mut stat = StreamTimeStat::build(2);
        stat.add_sample(1.0);
        assert_eq!(stat.get_mean(), 0.25);
        stat.add_sample(1.0);
        stat.add_sample(1.0);
        assert!(stat.get_mean() > 0.25);
    }
}

pub struct MicroTimer {
    last_time: u128,
    interval: u128,
}

impl MicroTimer {
    pub fn build(now: u128, interval: u128) -> MicroTimer {
        MicroTimer {
            last_time: now,
            interval: interval,
        }
    }
    pub fn set_interval(&mut self, interval: u128) -> () {
        self.interval = interval;
    }
    pub fn expired(&self, now: u128) -> bool {
        (self.last_time + self.interval) < now
    }
    pub fn reset(&mut self, now: u128) {
        self.last_time = now;
    }
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
        let mut mt = MicroTimer::build(now, 100);
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
