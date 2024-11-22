//! pitch detector used to extract primary frequency of incoming frames
//!
//! Used the rustfft to find the peak frequency in the inputs.  This is done by
//! - applying stack of 3 BiQuad LoPass filters
//! - downsample the incoming stream
//! - supply overalapping frame of data with raised cosine window to fft
//! - find magnitudes of largest frequency components
//! - parapolic interpolation to get better frequency resolution

use super::{biquad::{BiQuadFilter, FilterType}, moving_avg::MovingAverage, pitch_detector::PitchDetector};

const HIGHEST_NOTE: f32 = 400.0;

pub struct Tuner {
    pub enable: bool,
    pitch_detector: PitchDetector,
    note: f32,
    filter_stack: [BiQuadFilter; 2],
    avg: MovingAverage,
}

impl Tuner {
    pub fn new() -> Tuner {
        let mut tuner = Tuner {
            enable: false,
            pitch_detector: PitchDetector::new(),
            note: 0.0,
            filter_stack: [ BiQuadFilter::new(), BiQuadFilter::new() ],
            avg: MovingAverage::new(100),
        };
        for filter in &mut tuner.filter_stack {
            filter.init(
                FilterType::LowPass,
                1.1 * HIGHEST_NOTE as f64,
                1.0,
                0.707,
                48000.0,
            );
        }
        tuner
    }
    pub fn get_note(&mut self) -> f64 {
        self.avg.get_mean()
        // self.note as f64
    }

    pub fn add_samples(&mut self, input: &[f32]) -> () {
        let mut in_filtered: Vec<f32> = Vec::new();
        for v in input {
            let mut f = *v;
            for filter in &mut self.filter_stack {
                f = filter.get_sample(&f);
            }
            in_filtered.push(f);
        }

        let mut output: Vec<f32> = Vec::new();
        output.extend_from_slice(input);
        self.note = self.pitch_detector.do_compute(&in_filtered, &mut output);
        self.avg.add_sample(self.note as f64);
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
