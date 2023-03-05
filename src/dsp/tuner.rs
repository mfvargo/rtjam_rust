use num::Complex;
use rustfft::{algorithm::Radix4, Fft, FftDirection};

use super::{
    biquad::{BiQuadFilter, FilterType},
    smoothing_filter::SmoothingFilter,
};

const FFT_SIZE: usize = 256;
const DOWN_COUNT: usize = 24;

pub struct Tuner {
    pub enable: bool,
    pub freq: SmoothingFilter,
    filter_stack: [BiQuadFilter; 4],
    down_sample_count: usize,
    window: [f32; FFT_SIZE],
    fft_out: [Complex<f32>; FFT_SIZE],
    fft_bin: usize,
    fft: Radix4<f32>,
}

impl Tuner {
    pub fn new() -> Tuner {
        let mut tuner = Tuner {
            enable: false,
            freq: SmoothingFilter::build(0.01, 1000),
            filter_stack: [
                BiQuadFilter::new(),
                BiQuadFilter::new(),
                BiQuadFilter::new(),
                BiQuadFilter::new(),
            ],
            down_sample_count: 0,
            window: [0.0; FFT_SIZE],
            fft_out: [Complex { re: 0.0, im: 0.0 }; FFT_SIZE],
            fft_bin: 0,
            fft: Radix4::new(FFT_SIZE, FftDirection::Forward),
        };
        // Initialize filter stack
        for filter in &mut tuner.filter_stack {
            filter.init(FilterType::LowPass, 350.0, 1.0, 0.707, 48000.0);
        }
        // Build window
        for i in 0..FFT_SIZE - 1 {
            tuner.window[i] =
                0.5 * (1.0 - f32::cos(2.0 * std::f32::consts::PI / (FFT_SIZE as f32 - 1.0)));
        }
        tuner
    }
    pub fn get_note(&mut self) -> f64 {
        self.freq.get_last_output()
    }
    pub fn add_samples(&mut self, input: &[f32]) -> () {
        if self.enable {
            for samp in input {
                self.process_sample(*samp);
            }
        }
    }

    pub fn process_sample(&mut self, input: f32) -> () {
        let mut value = input;
        let mut batch_ready = false;

        // Run the filterstack to knock down f > 370 before the down sample
        for filter in &mut self.filter_stack {
            value = filter.get_sample(&value);
        }

        if self.down_sample_count % DOWN_COUNT == 0 {
            self.down_sample_count = 0;
            // put every 48th sample into the fft (downsample) but gain up for missing vals
            self.fft_out[self.fft_bin].re = value * 48.0 * self.window[self.fft_bin];
            self.fft_out[self.fft_bin].im = 0.0;
            self.fft_bin += 1;
            // set flag when we have filled up the fft buffer
            batch_ready = self.fft_bin % FFT_SIZE == 0;
        }
        self.down_sample_count += 1;

        // This is where we have a batch of values ready to calc.
        if batch_ready {
            // reset the bin index
            self.fft_bin = 0;

            // do the fft thing
            self.fft.process(&mut self.fft_out);
            // println!("fft output: {:?}", self.fft_out);

            // Calculate magnitudes of 1/4 of the results cause those are the only
            // meaningful values.  Higher vals are just harmonics
            let mut mags: [f32; FFT_SIZE / 4] = [0.0; FFT_SIZE / 4];
            for i in 0..mags.len() - 1 {
                mags[i] = f32::sqrt(
                    self.fft_out[i].re * self.fft_out[i].re
                        + self.fft_out[i].im * self.fft_out[i].im,
                );
            }

            // find the largest mag
            let mut max_idx = 0;
            let mut max_bin_value = mags[0];
            for i in 0..mags.len() - 1 {
                if mags[i] > max_bin_value {
                    max_bin_value = mags[i];
                    max_idx = i;
                }
            }
            // parabolic intepolation
            // p - peak location relative to maxbin (+/- 0.5 samples)
            // p = 0.5*((alpha-gamma)/(alpha-2*beta + gamma))
            // alpha -  bin to left of max bin
            // beta -  max value bin
            // gamma - bin to right of max bin

            // Make sure the bins don't over/underflow
            max_idx = usize::clamp(max_idx, 1, mags.len() - 2);
            let alpha = mags[max_idx - 1];
            let beta = mags[max_idx];
            let gamma = mags[max_idx + 1];
            // float p = 0.5*((alpha-gamma)/(alpha-2*beta + gamma));
            // note - test - should be equiv solution for parabola
            let p = (gamma - alpha) / (2.0 * (2.0 * beta - gamma - alpha));
            if !p.is_nan() && max_idx > 1 {
                self.freq
                    .get(((p + max_idx as f32) * 48_000.0 / (DOWN_COUNT * FFT_SIZE) as f32) as f64);
                // println!("detected: {}", self.freq.get_last_output());
            } else {
                self.freq.get(0.0);
            }
        }
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

        let mut input: Vec<f32> = vec![0.0; 8192 * 32];
        for i in 0..input.len() - 1 {
            input[i] = f32::sin(i as f32 * 2.0 * std::f32::consts::PI * 453.0 / 48_000.0);
        }
        while input.len() > 0 {
            tuner.add_samples(&input[0..128]);
            input.drain(0..128);
            println!("note: {}", tuner.get_note());
        }
    }
}
