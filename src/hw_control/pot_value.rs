use std::fmt;


pub struct PotValue {
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    name: String,
}

impl fmt::Display for PotValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:.2}", self.name, self.value)
    }
}

impl PotValue {
    pub fn new(name: &str, min: f64, max: f64, step: f64) -> Self {
        PotValue {
            value: 0.0,
            min,
            max,
            step,
            name: String::from(name),
        }
    }

    pub fn set_value(&mut self, value: f64) -> bool {
        let new_value = value.clamp(self.min, self.max);
        if (self.value - new_value).abs() > self.step {
            self.value = new_value;
            return true;
        }
        false
    }
    pub fn get_value(&self) -> f64 {
        self.value
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn build() {
        // You should be able to build a PotValue
        let pot = PotValue::new("test", 0.0, 1.0, 0.1);
        assert_eq!(pot.name, "test");
    }

    #[test]
    fn set_value() {
        // You should be able to set the value of a PotValue
        let mut pot = PotValue::new("test", 0.0, 1.0, 0.1);
        assert_eq!(pot.set_value(0.5), true);
        assert_eq!(pot.value, 0.5);
        assert_eq!(pot.set_value(0.5), false);
        assert_eq!(pot.value, 0.5);
    }

    #[test]
    fn set_value_out_of_bounds() {
        // You should be able to set the value of a PotValue
        let mut pot = PotValue::new("test", 0.0, 1.0, 0.1);
        assert_eq!(pot.set_value(2.0), true);
        assert_eq!(pot.value, 1.0);
    }
}
