//! base implementation of an overdrive pedal.  THe various overdrives are just skins on this base pedal
//!
//! comprised of two gain stages with a clipping function a number of filters pre/post clip
//! and an overall EQ at the end.
use crate::dsp::{
    biquad::{BiQuadFilter, FilterType},
    clip::{clip_sample, ClipType},
};

#[derive(ToPrimitive, FromPrimitive)]
pub enum HpfMode {
    Low,
    High,
}
pub struct DistortionBase {
    // These filters are private because they are called in this classes process and
    // setupFilters functions.  Derived classes do not need to access them.
    m_hpf1: BiQuadFilter, // pre-clipping high-pass filter
    m_hpf2: BiQuadFilter, // post-clip 1 high-pass filter
    m_hpf3: BiQuadFilter, // post-clip 2 high-pass filter

    m_lpf1: BiQuadFilter, // pre-clip 1 low-pass filter
    m_lpf2: BiQuadFilter, // post clip 1 low pass-filter
    m_lpf3: BiQuadFilter, // post clip 1 low pass-filter

    m_tone_bass: BiQuadFilter,   // bass control frequency
    m_tone_mid: BiQuadFilter,    // mid control frequency
    m_tone_treble: BiQuadFilter, // treble control frequency

    /// clip funciton type for stage 1
    pub clip1_type: ClipType,
    /// clip funciton type for stage 2
    pub clip2_type: ClipType,
    /// stage1 gain
    pub gain1: f32, // gain before clip functions
    /// stage2 gain
    pub gain2: f32, // gain before clip functions

    /// Number of stages of gain to apply (single/dual)
    pub num_stages: i32, // number of overdrive stages (single/dual)

    /// overall gain
    pub level: f32, // Overall level

    pub lpf1_freq: f64, // frequency of the first clip block lpf (fixed filter)
    pub lpf2_freq: f64, // frequency of the first clip block lpf (fixed filter)
    pub lpf3_freq: f64, // frequency of the first clip block lpf (fixed filter)

    pub hpf1_freq: f64, // high-pass filter 1 cutoff
    pub hpf2_freq: f64, // high-pass filter 1 cutoff
    pub hpf3_freq: f64, // high-pass filter 1 cutoff

    pub tone_bass_freq: f64,   // LPF cut-off frequency for tone control
    pub tone_mid_freq: f64,    // Peaking filter cut/boost frequency for tone control
    pub tone_mid_q: f64,       // Peaking filter Q
    pub tone_treble_freq: f64, // HPF cut-off frequency for tone control

    pub tone_bass_cut_boost: f64, // LPF cut-off frequency for tone control
    pub tone_mid_cut_boost: f64,  // HPF cut-off frequency for tone control
    pub tone_treble_cut_boost: f64, // HPF cut-off frequency for tone control

    pub dry_level: f32, // amount of dry to add in at end of chain
                        // (to model Klon type drives or add detail to high gain model)
}

impl DistortionBase {
    pub fn new() -> DistortionBase {
        let mut dist = DistortionBase {
            m_hpf1: BiQuadFilter::new(), // pre-clipping high-pass filter
            m_hpf2: BiQuadFilter::new(), // post-clip 1 high-pass filter
            m_hpf3: BiQuadFilter::new(), // post-clip 2 high-pass filter

            m_lpf1: BiQuadFilter::new(), // pre-clip 1 low-pass filter
            m_lpf2: BiQuadFilter::new(), // post clip 1 low pass-filter
            m_lpf3: BiQuadFilter::new(), // post clip 1 low pass-filter

            m_tone_bass: BiQuadFilter::new(), // bass control frequency
            m_tone_mid: BiQuadFilter::new(),  // mid control frequency
            m_tone_treble: BiQuadFilter::new(), // treble control frequency

            // type of clipping for stage 1
            clip1_type: ClipType::Soft, // clip funciton type for stage 1
            clip2_type: ClipType::Soft, // clip funciton type for stage 2

            gain1: 1.0, // gain before clip functions
            gain2: 1.0, // gain before clip functions

            num_stages: 2, // number of overdrive stages (single/dual)

            level: 1.0, // Overall level

            lpf1_freq: 250.0, // frequency of the first clip block lpf (fixed filter)
            lpf2_freq: 250.0, // frequency of the first clip block lpf (fixed filter)
            lpf3_freq: 250.0, // frequency of the first clip block lpf (fixed filter)

            hpf1_freq: 250.0, // high-pass filter 1 cutoff
            hpf2_freq: 30.0,  // high-pass filter 1 cutoff
            hpf3_freq: 30.0,  // high-pass filter 1 cutoff

            tone_bass_freq: 130.0,    // LPF cut-off frequency for tone control
            tone_mid_freq: 740.0,     // Peaking filter cut/boost frequency for tone control
            tone_mid_q: 0.9,          // Peaking filter Q
            tone_treble_freq: 2200.0, // HPF cut-off frequency for tone control

            tone_bass_cut_boost: 14.0, // LPF cut-off frequency for tone control
            tone_mid_cut_boost: 24.0,  // HPF cut-off frequency for tone control
            tone_treble_cut_boost: 10.0, // HPF cut-off frequency for tone control

            dry_level: 0.0, // amount of dry to add in at end of chain
        };
        dist.init();
        dist
    }

    pub fn init(&mut self) {
        self.m_hpf1
            .init(FilterType::HighPass, self.hpf1_freq, 1.0, 1.0, 48000.0);
        self.m_hpf2
            .init(FilterType::HighPass, self.hpf2_freq, 1.0, 1.0, 48000.0);
        self.m_hpf3
            .init(FilterType::HighPass, self.hpf3_freq, 1.0, 1.0, 48000.0);
        self.m_lpf1
            .init(FilterType::LowPass, self.lpf1_freq, 1.0, 1.0, 48000.0);
        self.m_lpf2
            .init(FilterType::LowPass, self.lpf2_freq, 1.0, 1.0, 48000.0);
        self.m_lpf3
            .init(FilterType::LowPass, self.lpf3_freq, 1.0, 1.0, 48000.0);

        self.m_tone_bass.init(
            FilterType::LowShelf,
            self.tone_bass_freq,
            self.tone_bass_cut_boost,
            0.707,
            48000.0,
        );
        self.m_tone_mid.init(
            FilterType::Peaking,
            self.tone_mid_freq,
            self.tone_mid_cut_boost,
            self.tone_mid_q,
            48000.0,
        );
        self.m_tone_treble.init(
            FilterType::HighShelf,
            self.tone_treble_freq,
            self.tone_treble_cut_boost,
            0.707,
            48000.0,
        );
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) -> () {
        // Implement the delay
        let mut i = 0;
        for samp in input {
            // HPF in chain - determines amount of low-freqs sent to clipper block
            // use F~140 Hz for full-range. F=720 for Tubescreamer type distorion
            let mut value = self.m_hpf1.get_sample(samp);
            value = self.m_lpf1.get_sample(&value);

            // Stage 1 - first clipper (set to emulate op-amp clipping before diodes)
            value = clip_sample(&self.clip1_type, value * self.gain1); // clip signal

            // filter out higher-order harmonics
            value = self.m_lpf2.get_sample(&value);
            value = self.m_hpf2.get_sample(&value);

            // Stage 2 - second clipper option if m_stages == 2
            if self.num_stages == 2 {
                value = self.m_hpf2.get_sample(&value);
                value = clip_sample(&self.clip2_type, value * self.gain2);
                value = self.m_lpf3.get_sample(&value); // filter out higher-order harmonics
                value = self.m_hpf3.get_sample(&value);
            }
            // Stage 3 - Tone control - 3 band EQ - low shelf, mid cut/boost, high shelf
            // Baxandall type w/ mid control
            //
            value = self.m_tone_bass.get_sample(&value);
            value = self.m_tone_mid.get_sample(&value);
            value = self.m_tone_treble.get_sample(&value);

            // sum in some dry level for detail (to model Klon and similar pedals)

            output[i] = (value * self.level) + (self.dry_level * samp);
            i += 1;
        }
    }
}

#[cfg(test)]
mod test_delay_base {

    use super::*;

    #[test]
    fn can_build_and_run() {
        let mut dist = DistortionBase::new();
        dist.init();
        let input = [1.0; 128];
        let mut output = [1.0; 128];
        dist.process(&input, &mut output);
        println!("out: {:?}", output);
    }
}
