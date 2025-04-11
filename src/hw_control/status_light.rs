use rppal::gpio::{Gpio, OutputPin};
use crate::common::{box_error::BoxError, get_micro_time, stream_time_stat::MicroTimer};
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

#[derive(Serialize, Deserialize, Debug)]
pub enum Color {
    Black,
    Green,
    Orange,
    Red,
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct LightMessage {
//     pub input_one: f64,
//     pub input_two: f64,
//     pub status: Color,
//     pub blink: bool,
// }

#[derive(Serialize, Deserialize, Debug)]
pub enum HardwareMessage {
    LightMessage {
        input_one: f64,
        input_two: f64,
        status: Color,
        blink: bool,
    },
    GainMessage {
        input_1_gain: f64,
        input_2_gain: f64,
        headphone_gain: f64
    },
}
pub struct StatusLight {
    red_pin: OutputPin,  // Pin for the red led
    green_pin: OutputPin, // Pin for the green led
    light_state: bool,  // State of the light (used for blinking)
    blink_timer: MicroTimer, // Blink timer
}

const BLINK_TIME: u128 = 1_000_000;  // One second blink interval

impl StatusLight {
    pub fn new(light_type: StatusFunction) -> Result<StatusLight, BoxError> {
        let timer = MicroTimer::new(get_micro_time(), BLINK_TIME);
        match light_type {
            StatusFunction::InputOne => {
                Ok(StatusLight {
                    red_pin: Gpio::new()?.get(6)?.into_output(),
                    green_pin: Gpio::new()?.get(5)?.into_output(),
                    light_state: true,
                    blink_timer: timer,
                })
            }
            StatusFunction::InputTwo => {
                Ok(StatusLight {
                    red_pin: Gpio::new()?.get(8)?.into_output(),
                    green_pin: Gpio::new()?.get(7)?.into_output(),
                    light_state: true,
                    blink_timer: timer,
                })
            }
            StatusFunction::Status => {
                Ok(StatusLight {
                    red_pin: Gpio::new()?.get(23)?.into_output(),
                    green_pin: Gpio::new()?.get(24)?.into_output(),
                    light_state: true,
                    blink_timer: timer,
                })
            }
        }
    }
    pub fn set(&mut self, color: Color, blink: bool) -> () {

        let mut col = color;

        // do this if we are blinking
        if blink {
            let now = get_micro_time();
            if self.blink_timer.expired(now) {
                // When the blink timer has expired
                self.blink_timer.reset(now);  // reset the timer
                self.light_state = !self.light_state; // toggle the light_state
            }
            if !self.light_state {
                col = Color::Black;  // Turn of the light 
            }
    }
        // Do the right pins for the color
        match col {
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
        if power < -59.9 {
            self.set(Color::Black, false);
        } else if power < -35.5 {
            self.set(Color::Green, false);
        } else if power < -20.0 {
            self.set(Color::Orange, false);
        } else {
            self.set(Color::Red, false);
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
                light.set(Color::Orange, false);
                light.power(-10.0);
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }}
