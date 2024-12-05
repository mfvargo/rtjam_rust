//! Variable length delay with gain
use num::{Float, FromPrimitive, Zero};

pub struct DelayLine<T> {
    gain: T,
    delay_line: Vec<T>,
    length: usize,
}

impl<T: Float + FromPrimitive> DelayLine<T> {
    pub fn one() -> T {
        T::from_i64(1).unwrap()
    }
    pub fn new() -> DelayLine<T> {
        DelayLine {
            gain: Self::one(),
            delay_line: Vec::<T>::new(),
            length: 0
        }
    }
    pub fn init(&mut self, delay_length: usize, gain: T) -> () {
        self.gain = gain;
        self.set_length(delay_length);
    }
    pub fn set_gain(&mut self, gain: T) -> () {
        self.gain = gain;
    }
    pub fn get_gain(&self) -> T {
        self.gain
    }
    pub fn get_length(&self) -> usize {
        self.length
    }
    pub fn set_length(&mut self, new_length: usize) -> () {
        self.length = new_length;
    }
    pub fn get_sample(&mut self, input: T) -> T {
        // append the sample
        self.delay_line.push(input);
        // in case the line is not full
        let mut output = Zero::zero();
        while self.delay_line.len() > self.length {
            output = self.delay_line.remove(0);
        }
        output * self.gain
    }
}

#[cfg(test)]
mod test_allpass_delay {
    use super::*;

    #[test]
    fn can_build() {
        let mut dl = DelayLine::new();
        dl.init(2, 1.2);
    }

    #[test]
    fn can_init_and_run() {
        let mut dl = DelayLine::new();
        dl.init(2, 0.5);
        let input = [1.0, 1.0, 1.0];
        let mut output: Vec<f32> = vec![];
        for samp in input {
            output.push(dl.get_sample(samp));
        }
        assert_eq!(output, [0.0, 0.0, 0.5])
    }

    #[test]
    fn can_be_adjusted() {
        let mut dl = DelayLine::new();
        dl.init(2, 0.5);
        assert_eq!(dl.get_gain(), 0.5);
        assert_eq!(dl.get_length(), 2);
        dl.set_gain(0.8);
        assert_eq!(dl.get_gain(), 0.8);
        dl.set_length(10);
        assert_eq!(dl.get_length(), 10);
        let input = [1.0; 8];
        let mut output: Vec<f32> = vec![];
        for samp in input {
            output.push(dl.get_sample(samp));
        }
    }

}
