use core::fmt;

use pedal_board::dsp::smoothing_filter::SmoothingFilter;

pub struct AdcValue {
    value: u16,
    filter: SmoothingFilter,
}

impl fmt::Display for AdcValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ADC Value: {}", self.value)
    }
}

impl AdcValue {
    pub fn new() -> Self {
        AdcValue {
            value: 0,
            filter: SmoothingFilter::build(0.1, 200.0),
        }
    }

    pub fn set_value(&mut self, value: u16) -> () {
        // Apply the smoothing filter to the new value
        let smoothed_value = self.filter.get(value as f64);
        // Update the value
        self.value = smoothed_value as u16;
    }

    pub fn get_value(&self) -> u16 {
        self.value
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn build() {
        // You should be able to build a PotValue
        let adc = AdcValue::new();
        assert_eq!(adc.value, 0);
    }

    #[test]
    fn set_value() {
        // You should be able to set the value of a PotValue
        let mut adc = AdcValue::new();
        adc.set_value(1000);
        assert!(adc.value > 0);
        assert!(adc.value < 1000);
    }
}