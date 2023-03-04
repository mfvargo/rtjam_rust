use std::fmt;

use crate::{
    dsp::{
        biquad::{BiQuadFilter, FilterType},
        low_freq_osc::{LowFreqOsc, WaveShape},
    },
    utils::to_lin,
};

#[derive(ToPrimitive, FromPrimitive)]
pub enum DelayMode {
    Digital,
    Analog,
    HighPass,
}
pub struct DelayBase {
    m_osc: LowFreqOsc<f32>,
    feedback_filter: BiQuadFilter,
    delay_buffer: Vec<f32>,
    buffer_depth: i32,
    write_index: i32,
    pub current_delay_time: f32,
    pub feedback: f32,
    pub level: f32,
    pub drift: f32,
    pub drift_rate: f32,
    pub gain: f32,
    pub delay_mode: DelayMode,
    sample_rate: f32,
}

impl DelayBase {
    pub fn new() -> DelayBase {
        let mut delay = DelayBase {
            m_osc: LowFreqOsc::new(),
            feedback_filter: BiQuadFilter::new(),
            delay_buffer: vec![0.0; 96000],
            buffer_depth: 100,
            current_delay_time: 0.250,
            write_index: 0,
            feedback: 0.1,
            level: 0.5,
            drift: to_lin(-42.0) as f32,
            drift_rate: 1.4,
            gain: 1.0,
            delay_mode: DelayMode::Digital,
            sample_rate: 48_000.0,
        };
        delay.init();
        delay
    }

    pub fn init(&mut self) {
        match self.delay_mode {
            DelayMode::Digital => {
                self.feedback_filter.init(
                    FilterType::LowPass,
                    10000.0,
                    1.0,
                    1.0,
                    self.sample_rate as f64,
                );
            }
            DelayMode::Analog => {
                self.feedback_filter.init(
                    FilterType::LowPass,
                    1250.0,
                    1.0,
                    1.0,
                    self.sample_rate as f64,
                );
            }
            DelayMode::HighPass => {
                self.feedback_filter.init(
                    FilterType::HighPass,
                    1250.0,
                    1.0,
                    1.0,
                    self.sample_rate as f64,
                );
            }
        }
        self.m_osc.init(
            WaveShape::Sine,
            self.drift_rate,
            self.drift,
            self.sample_rate as f32,
        );
        self.buffer_depth =
            ((1.0 + self.drift) * self.current_delay_time * self.sample_rate) as i32;
    }
    //  Digital Delay Effect - Signal Flow Diagram
    //
    //  Delay with modulation and filter.
    //  LPF for analog delay simulation
    //  HPF for "thinning delay"
    //
    //          ┌───────────────────────────────────────────┐
    //          │                                           │
    //          │             ┌────────────┐                ▼
    //          │    ┌────┐   │            │    ┌─────┐   ┌────┐
    //  Input───┴───►│Sum ├──►│   Delay    ├─┬─►│Level├──►│Sum ├───► Output
    //               └────┘   │            │ │  └─────┘   └────┘
    //                 ▲      └────────────┘ │
    //                 │            ▲        │
    //                 │            │        │
    //              ┌──┴───┐     ┌──┴──┐     │
    //      LPF/HPF │Filter│     │ Mod │     │
    //              └──────┘     └─────┘     │
    //                 ▲                     │
    //                 │        ┌────────┐   │
    //                 └────────┤Feedback│◄──┘
    //                          └────────┘
    //                            0-1.2
    //
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) -> () {
        // Implement the delay
        let mut i = 0;
        for samp in input {
            // pointer arithmetic for buffer wrap
            self.write_index += 1;
            self.write_index %= self.buffer_depth;

            // clamp the write index so it can't blow up the buffer
            self.write_index = self.write_index.clamp(0, 96000 - 1);

            // Use the low freq osc to modulate the delay
            let mut read_index = self.write_index
                - ((1.0 + self.m_osc.get_sample()) * self.current_delay_time * self.sample_rate)
                    as i32;

            if read_index < 0 {
                read_index += self.buffer_depth;
            }
            // pointer arithmetic for buffer wrap
            read_index %= self.buffer_depth;

            // clamp the read index so it can't blow up the buffer
            read_index = read_index.clamp(0, 96000 - 1);

            // return original plus delay
            output[i] = self.gain * (samp + self.delay_buffer[read_index as usize] * self.level);
            i += 1;

            // add feedback to the buffer
            self.delay_buffer[self.write_index as usize] = samp
                + (self
                    .feedback_filter
                    .get_sample(&self.delay_buffer[read_index as usize])
                    * self.feedback);
        }
    }
}

impl fmt::Display for DelayBase {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ delay: {}, feedbac: {}, level: {}, drift: {}, rate: {}, depth: {} }}",
            self.current_delay_time,
            self.feedback,
            self.level,
            self.drift,
            self.drift_rate,
            self.buffer_depth
        )
    }
}
#[cfg(test)]
mod test_delay_base {

    use super::*;

    #[test]
    fn can_build_and_run() {
        let mut delay = DelayBase::new();
        delay.init();
        let input = [1.0; 128];
        let mut output = [1.0; 128];
        delay.process(&input, &mut output);
        println!("out: {:?}", output);
    }
}
