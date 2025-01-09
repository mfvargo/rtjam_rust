//!
//! object to provide a click track on beat changes
//! 

use pedal_board::dsp::low_freq_osc::{LowFreqOsc, WaveShape};

pub struct ClickTrack {
    gain: f32,
    beat: u8,
    tic: Vec<f32>,
    toc: Vec<f32>,
    idx: usize,
    mute: bool,
}

const CLICK_SIZE: usize = 4800;

impl ClickTrack {
    pub fn new() -> ClickTrack {
        let mut rval = ClickTrack {
            gain: 1.0,
            mute: true,
            beat: 0,
            tic: vec![],
            toc: vec![],
            idx: 0,
        };
        let mut i = 0;
        let mut osc: LowFreqOsc<f32> = LowFreqOsc::new();
        osc.init(WaveShape::Sine, 330.0, 1.0, 48_000.0);
        while i < CLICK_SIZE {
            rval.tic.push(osc.get_sample());
            i += 1;
        }
        i = 0;
        osc.init(WaveShape::Sine, 300.0, 0.7, 48_000.0);
        while i < CLICK_SIZE {
            rval.toc.push(osc.get_sample());
            i += 1;
        }
        rval
    }
    pub fn get_gain(&self) -> f64 {
        self.gain as f64
    }
    pub fn get_mute(&self) -> bool {
        self.mute
    }
    pub fn set_gain(&mut self, gain: f64) -> () {
        self.gain = gain as f32;
    }
    pub fn set_mute(&mut self, mute: bool) -> () {
        self.mute = mute;
    }
    pub fn mix_into(&mut self, beat: u8, out_a: &mut [f32], out_b: &mut [f32]) -> () {
        // this is where we mix in the click
        if beat != self.beat {
            self.beat = beat;
            self.idx = 0;
        }
        if !self.mute && self.idx < CLICK_SIZE - out_a.len() {
            let mut i: usize = 0;
            while i < out_a.len() {
                if beat == 0 {
                    out_a[i] += self.gain * self.tic[self.idx+i];
                    out_b[i] += self.gain * self.tic[self.idx+i];
                } else {
                    out_a[i] += self.gain * self.toc[self.idx+i];
                    out_b[i] += self.gain * self.toc[self.idx+i];
                }
                i += 1;
            }
            self.idx += out_a.len();
        }
    }
}