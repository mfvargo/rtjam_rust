use super::{
    biquad::{BiQuadFilter, FilterType},
    smoothing_filter::SmoothingFilter,
};
use pitch_detector::{note::detect_note, pitch::HannedFftDetector};

const FFT_SIZE: usize = 256;
const DOWN_COUNT: usize = 48;

pub struct Tuner {
    pub enable: bool,
    pub freq: SmoothingFilter<f64>,
    pub last_note: f64,
    filter_stack: [BiQuadFilter; 4],
    down_sample_count: usize,
    samples: [f64; FFT_SIZE],
    sample_idx: usize,
    detector: HannedFftDetector,
}

impl Tuner {
    pub fn new() -> Tuner {
        let mut tuner = Tuner {
            enable: false,
            freq: SmoothingFilter::build(0.01, (48_000 / DOWN_COUNT) as f64),
            filter_stack: [
                BiQuadFilter::new(),
                BiQuadFilter::new(),
                BiQuadFilter::new(),
                BiQuadFilter::new(),
            ],
            down_sample_count: 0,
            samples: [0.0; FFT_SIZE],
            sample_idx: 0,
            detector: HannedFftDetector::default(),
            last_note: 0.0,
        };
        // Initialize filter stack
        for filter in &mut tuner.filter_stack {
            filter.init(FilterType::LowPass, 350.0, 1.0, 0.707, 48000.0);
        }
        tuner
    }
    pub fn get_note(&mut self) -> f64 {
        // self.last_note
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

        // Run the filterstack to knock down f > 350 before the down sample
        for filter in &mut self.filter_stack {
            value = filter.get_sample(&value);
        }

        if self.down_sample_count % DOWN_COUNT == 0 {
            self.down_sample_count = 0;
            // put every DOWN_COUNT sample into the detector input but gain up for missing vals
            self.samples[self.sample_idx] = value as f64 * DOWN_COUNT as f64;
            self.sample_idx += 1;
            // set flag when we have filled up the fft buffer
            batch_ready = self.sample_idx % FFT_SIZE == 0;
        }
        self.down_sample_count += 1;

        // This is where we have a batch of values ready to calc.
        if batch_ready {
            // reset the bin index
            self.sample_idx = 0;

            match detect_note(
                &self.samples,
                &mut self.detector,
                48_000.0 / DOWN_COUNT as f64,
            ) {
                Some(res) => {
                    self.freq.get(res.note_freq);
                    self.last_note = res.note_freq;
                }
                None => {
                    self.freq.get(0.0);
                    self.last_note = 0.0;
                }
            };
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
