use crate::utils::{get_frame_power_in_db, to_db};

use super::{peak_detector::PeakDetector, smoothing_filter::SmoothingFilter};

///! Calculates the peak and avg power in dB
pub struct PowerMeter {
    peak: PeakDetector,
    avg: SmoothingFilter,
    last_peak: f64,
    last_avg: f64,
}

impl PowerMeter {
    pub fn new() -> PowerMeter {
        PowerMeter {
            peak: PeakDetector::build(0.01, 0.1, 2666),
            avg: SmoothingFilter::build(0.01, 2666),
            last_peak: 0.0,
            last_avg: 0.0,
        }
    }
    pub fn get_peak(&self) -> f64 {
        self.last_peak
    }
    pub fn get_avg(&self) -> f64 {
        self.last_avg
    }
    pub fn add_frame(&mut self, data: &[f32], gain: f32) -> () {
        let p = get_frame_power_in_db(data) + to_db(gain);
        self.last_peak = self.peak.get(p as f64);
        self.last_avg = self.avg.get(p as f64);
    }
}
