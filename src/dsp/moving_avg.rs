use serde::{ Deserialize, Serialize };




#[derive(Debug, Deserialize, Serialize)]
pub struct MovingAverage {
    window: usize,
    total: f64,
    samples: Vec<f64>,
}

impl MovingAverage {
    pub fn new(window_size: usize) -> MovingAverage {
        MovingAverage {
            window: window_size,
            total: 0.0,
            samples: vec![0.0; window_size],
        }
    }
    pub fn get_mean(&self) -> f64 {
        self.total / self.window as f64
    }

    pub fn get_total(&self) -> f64 {
        self.total
    }

    pub fn get_window(&self) -> usize {
        self.window
    }

    pub fn add_sample(&mut self, v: f64) -> () {
        self.total += v;
        self.samples.push(v);
        self.total -= self.samples.remove(0);
    }
}

#[cfg(test)]
mod test_moving_average {
    use super::*;

    #[test]
    fn build() {
        let stat = MovingAverage::new(5);
        assert_eq!(stat.get_mean(), 0.0);
    }
    #[test]
    fn add_sample() {
        let mut stat = MovingAverage::new(2);
        stat.add_sample(1.0);
        assert_eq!(stat.get_mean(), 0.5);
        stat.add_sample(1.0);
        assert!(stat.get_mean() > 0.99999);
    }
}