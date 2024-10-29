use rppal::gpio::{Gpio, OutputPin};
use crate::common::box_error::BoxError;
use serde::{Deserialize, Serialize};

pub fn has_lights() -> bool {
    match Gpio::new() {
        Ok(_) => {
            true
        }
        Err(e) => {
            dbg!(e);
            false
        }
    }
}

pub enum StatusFunction {
    InputOne,
    InputTwo,
    Status,
}

#[derive(Serialize, Deserialize)]
pub enum Color {
    Black,
    Green,
    Orange,
    Red,
}

#[derive(Serialize, Deserialize)]
pub struct LightMessage {
    pub input_one: f64,
    pub input_two: f64,
    pub status: Color,
}
pub struct StatusLight {
    red_pin: OutputPin,
    green_pin: OutputPin,
}

impl StatusLight {
    pub fn new(light_type: StatusFunction) -> Result<StatusLight, BoxError> {
        match light_type {
            StatusFunction::InputOne => {
                Ok(StatusLight {
                    red_pin: Gpio::new()?.get(6)?.into_output(),
                    green_pin: Gpio::new()?.get(5)?.into_output(),
                })
            }
            StatusFunction::InputTwo => {
                Ok(StatusLight {
                    red_pin: Gpio::new()?.get(8)?.into_output(),
                    green_pin: Gpio::new()?.get(7)?.into_output(),
                })
            }
            StatusFunction::Status => {
                Ok(StatusLight {
                    red_pin: Gpio::new()?.get(23)?.into_output(),
                    green_pin: Gpio::new()?.get(24)?.into_output(),
                })
            }
        }
    }
    pub fn set(&mut self, color: Color) -> () {
        match color {
            Color::Black => {
                self.red_pin.set_low();
                self.green_pin.set_low();
            }
            Color::Green => {
                self.red_pin.set_low();
                self.green_pin.set_high();
            }
            Color::Orange => {
                self.red_pin.set_high();
                self.green_pin.set_high();
            }
            Color::Red => {
                self.red_pin.set_high();
                self.green_pin.set_low();
            }
        }
    }

    pub fn power(&mut self, power: f64) -> () {

        if power < -32.0 {
            self.set(Color::Black);
        } else if power < -29.5 {
            self.set(Color::Green);
        } else if power < -24.0 {
            self.set(Color::Orange);
        } else {
            self.set(Color::Red);
        }
    }
}

#[cfg(test)]
mod test_lights {
    use super::*;

    #[test]
    fn toggle() {
        match StatusLight::new(StatusFunction::InputOne) {
            Ok(mut light) => {
                light.set(Color::Orange);
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }}
