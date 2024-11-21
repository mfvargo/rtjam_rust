//! pitch detector used to extract primary frequency of incoming frames
//!
//! Used the rustfft to find the peak frequency in the inputs.  This is done by
//! - applying stack of 3 BiQuad LoPass filters
//! - downsample the incoming stream
//! - supply overalapping frame of data with raised cosine window to fft
//! - find magnitudes of largest frequency components
//! - parapolic interpolation to get better frequency resolution

use crate::common::stream_time_stat::StreamTimeStat;

use super::pitch_detector::PitchDetector;

pub struct Tuner {
    pub enable: bool,
    pitch_detector: PitchDetector,
    stats: StreamTimeStat,
}

impl Tuner {
    pub fn new() -> Tuner {
        Tuner {
            enable: false,
            pitch_detector: PitchDetector::new(),
            stats: StreamTimeStat::new(50),
        }
    }
    pub fn get_note(&mut self) -> f64 {
        self.stats.get_mean()
    }

    pub fn add_samples(&mut self, input: &[f32]) -> () {
        let mut output: Vec<f32> = Vec::new();
        output.extend_from_slice(input);

        let freq = self.pitch_detector.do_compute(&input, &mut output);
        self.stats.add_sample(freq as f64);
    }

}

#[cfg(test)]
mod test_tuner {
    use super::*;

    #[test]
    fn detect_pitch() {
        let mut tuner = Tuner::new();
        tuner.enable = true;

        assert_eq!(tuner.get_note(), 0.0);

        let mut input: Vec<f32> = vec![0.0; 1024 * 32];
        for i in 0..input.len() {
            input[i] = f32::sin(i as f32 * 2.0 * std::f32::consts::PI * 453.0 / 48_000.0);
        }
        while input.len() > 0 {
            tuner.add_samples(&input[0..128]);
            input.drain(0..128);
            println!("note: {}", tuner.get_note());
        }
    }
}
