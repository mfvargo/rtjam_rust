//! Variable length delay with gain
use num::{Float, FromPrimitive, Zero};

pub struct AllpassDelay<T> {
    gain: T,
    delay_line: Vec<T>,
    delay_length: usize,
    index: usize,
}

impl<T: Float + FromPrimitive> AllpassDelay<T> {
    pub fn one() -> T {
        T::from_i64(1).unwrap()
    }
    pub fn new() -> AllpassDelay<T> {
        AllpassDelay {
            gain: Self::one(),
            delay_line: vec![Zero::zero()],
            delay_length: 1,
            index: 0,
        }
    }
    pub fn init(&mut self, delay_length: usize, gain: T) -> () {
        self.gain = gain;
        self.index = 0;
        self.delay_length = delay_length;
        self.delay_line = vec![Zero::zero(); self.delay_length];
    }
    pub fn get_sample(&mut self, input: T) -> T {
        let mut output = self.delay_line[self.index];
        let delay_in = input + output * self.gain;
        self.delay_line[self.index] = delay_in; // write to delay line - new delay sample
        output = output + delay_in * (-Self::one() * self.gain); // ap out = sum of delay out and ff path
        self.index += 1;
        self.index %= self.delay_length; // wrap the index around the buffer
        output
    }
}

#[cfg(test)]
mod test_allpass_delay {
    use super::*;

    #[test]
    fn can_build() {
        let mut ap = AllpassDelay::new();
        ap.init(2, 1.2);
    }

    #[test]
    fn can_init_and_run() {
        let mut ap = AllpassDelay::new();
        ap.init(2, 0.5);
        let input = [1.0; 8];
        let mut output: Vec<f32> = vec![];
        for samp in input {
            output.push(ap.get_sample(samp));
        }
        println!("output: {:?}", output);
    }
}
