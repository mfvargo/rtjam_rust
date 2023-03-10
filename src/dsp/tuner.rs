use num::Complex;
use rustfft::{algorithm::Radix4, Fft, FftDirection};

use super::biquad::{BiQuadFilter, FilterType};

const FFT_SIZE: usize = 1024;
const FFT_OVERLAP: usize = 256;
const DOWN_COUNT: usize = 12;
const HIGHEST_NOTE: f32 = 400.0;
const LOWEST_NOTE: f32 = 27.5;

pub struct Tuner {
    pub enable: bool,
    pub last_freq: f32,
    filter_stack: [BiQuadFilter; 4],
    down_sample_count: usize,
    window: [f32; FFT_SIZE],
    fft_in: Vec<f32>,
    fft_out: [Complex<f32>; FFT_SIZE],
    fft: Radix4<f32>,
}

impl Tuner {
    pub fn new() -> Tuner {
        let mut tuner = Tuner {
            enable: false,
            last_freq: 0.0,
            filter_stack: [
                BiQuadFilter::new(),
                BiQuadFilter::new(),
                BiQuadFilter::new(),
                BiQuadFilter::new(),
            ],
            down_sample_count: 0,
            window: [0.0; FFT_SIZE],
            fft_in: vec![0.0; FFT_SIZE],
            fft_out: [Complex { re: 0.0, im: 0.0 }; FFT_SIZE],
            fft: Radix4::new(FFT_SIZE, FftDirection::Forward),
        };
        // Initialize filter stack
        for filter in &mut tuner.filter_stack {
            filter.init(
                FilterType::LowPass,
                1.1 * HIGHEST_NOTE as f64,
                1.0,
                0.707,
                48000.0,
            );
        }
        // Build window (raised Cosine)
        for i in 0..FFT_SIZE - 1 {
            tuner.window[i] =
                0.5 * (1.0 - f32::cos(2.0 * std::f32::consts::PI / (FFT_SIZE as f32 - 1.0)));
        }
        tuner
    }
    pub fn get_note(&mut self) -> f64 {
        self.last_freq as f64
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

        // Extract note (only does something if we have enough samples)
        self.extract_note();

        // Run the filterstack to knock down f > 350 before the down sample
        for filter in &mut self.filter_stack {
            value = filter.get_sample(&value);
        }

        if self.down_sample_count % DOWN_COUNT == 0 {
            // put every DOWN_COUNT sample into the fft (downsample)
            self.down_sample_count = 0;
            self.fft_in.push(value * 100.0 * DOWN_COUNT as f32);
        }
        self.down_sample_count += 1;
    }

    fn extract_note(&mut self) -> () {
        // don't process until we have enough data
        if self.fft_in.len() < FFT_SIZE {
            return;
        }
        // Move fft_in via the window
        for i in 0..self.fft_in.len() {
            self.fft_out[i].re = self.fft_in[i] * self.window[i];
            self.fft_out[i].im = 0.0;
        }
        self.fft_in.drain(0..FFT_OVERLAP);
        // do the fft thing
        self.fft.process(&mut self.fft_out);

        // Calculate magnitudes of 1/4 of the results cause those are the only
        // meaningful values.  Higher vals are just harmonics
        let mut mags: [f32; FFT_SIZE / 4] = [0.0; FFT_SIZE / 4];
        for i in 0..mags.len() {
            mags[i] = f32::sqrt(
                self.fft_out[i].re * self.fft_out[i].re + self.fft_out[i].im * self.fft_out[i].im,
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
        // let p = 0.5 * ((alpha - gamma) / (alpha - 2.0 * beta + gamma));
        let p = (gamma - alpha) / (2.0 * (2.0 * beta - gamma - alpha));
        if !p.is_nan() && max_idx > 1 {
            let freq_det = (p + max_idx as f32) * 48_000.0 / (DOWN_COUNT * FFT_SIZE) as f32;
            // println!(
            //     "max_idx: {}, freq: {:.2}, [{:.2}, {:.2}, {:.2}]",
            //     max_idx, freq_det, alpha, beta, gamma
            // );
            if freq_det > LOWEST_NOTE && freq_det < HIGHEST_NOTE {
                self.last_freq = freq_det;
            }
        } else {
            self.last_freq = 0.0;
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
