use num::{Float, FromPrimitive, Zero};

pub enum WaveShape {
    Sine,
    Square,
    Ramp,
}

pub struct LowFreqOsc<T> {
    shape: WaveShape,
    amp: T,
    phase_inc: T,
    phase: T,
    pi: T,
    two_pi: T,
}

impl<T: Float + FromPrimitive> LowFreqOsc<T> {
    pub fn new() -> LowFreqOsc<T> {
        LowFreqOsc {
            shape: WaveShape::Sine,
            amp: T::from_f64(1.0).unwrap(),
            phase_inc: T::from_f64(0.01).unwrap(),
            phase: Zero::zero(),
            pi: T::from_f64(std::f64::consts::PI).unwrap(),
            two_pi: T::from_f64(std::f64::consts::PI * 2.0).unwrap(),
        }
    }
    pub fn init(&mut self, shape: WaveShape, freq: T, amp: T, sample_rate: T) -> () {
        self.shape = shape;
        self.phase_inc = self.two_pi * freq / sample_rate;
        self.amp = amp;
    }
    pub fn get_sample(&mut self) -> T {
        let val = match self.shape {
            WaveShape::Sine => self.amp * T::sin(self.phase),
            WaveShape::Square => {
                if self.phase < self.pi {
                    self.amp
                } else {
                    -self.amp
                }
            }
            WaveShape::Ramp => self.amp * self.phase / self.two_pi,
        };
        self.phase = self.phase + self.phase_inc;
        if self.phase >= self.two_pi {
            self.phase = Zero::zero();
        }
        val
    }
}

#[cfg(test)]
pub mod test_low_freq_osc {
    use super::*;

    #[test]
    fn can_make_waves() {
        let mut osc: LowFreqOsc<f32> = LowFreqOsc::new();
        let mut output: Vec<f32> = vec![0.0; 10];
        // Sine wave
        osc.init(WaveShape::Sine, 1000.0, 1.0, 4000.0);
        for i in 0..output.len() {
            output[i] = osc.get_sample();
        }
        println!("sine: {:?}", output);
        // Square wave
        osc.init(WaveShape::Square, 1000.0, 1.0, 4000.0);
        for i in 0..output.len() {
            output[i] = osc.get_sample();
        }
        println!("square: {:?}", output);
        // Ramp wave
        osc.init(WaveShape::Ramp, 1000.0, 1.0, 4000.0);
        for i in 0..output.len() {
            output[i] = osc.get_sample();
        }
        println!("ramp: {:?}", output);
    }
}
