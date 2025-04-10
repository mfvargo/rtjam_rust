use core::fmt;

use pedal_board::dsp::smoothing_filter::SmoothingFilter;

pub struct AdcValue {
    value: f64,
    full_scale: f64,
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
            value: 0.0,
            full_scale: 4096.0,
            filter: SmoothingFilter::build(0.1, 200.0),
        }
    }

    pub fn set_value(&mut self, value: f64) -> () {
        // Scale the adc value to the 12 bit adc value
        // Apply the smoothing filter to the new value
        self.value = self.filter.get(value/self.full_scale);
    }

    pub fn get_value(&self) -> f64 {
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
        assert_eq!(adc.value, 0.0);
    }

    #[test]
    fn set_value() {
        // You should be able to set the value of a PotValue
        let mut adc = AdcValue::new();
        adc.set_value(1.00);
        println!("ADC Value: {}", adc);
        assert!(adc.value > 0.0);
        assert!(adc.value < 1.0);
    }
}