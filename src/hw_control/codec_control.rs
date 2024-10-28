use std::thread::sleep;
use std::time::Duration;

use rppal::i2c::I2c;
use rppal::gpio::Gpio;
use crate::common::box_error::BoxError;

// Helper functions to encode and decode binary-coded decimal (BCD) values.
fn bcd2dec(bcd: u8) -> u8 {
    (((bcd & 0xF0) >> 4) * 10) + (bcd & 0x0F)
}

fn dec2bcd(dec: u8) -> u8 {
    ((dec / 10) << 4) | (dec % 10)
}


pub struct CodecControl {
    i2c_int: I2c,
}

impl CodecControl {
    pub fn new() -> Result<CodecControl, BoxError> {

        let mut codec = I2c::new()?;
        dbg!(&codec);

        // reset the code
        let mut pin = Gpio::new()?.get(17)?.into_output();
        pin.set_low();
        // Sleep for 200msec
        sleep(Duration::new(0, 200_000_000));
        pin.set_high();

        // Let the thing come up
        sleep(Duration::new(0, 200_000_000));

        codec.set_slave_address(0x18)?;

        // Set Reg 0 - select Page 0
        let buf: [u8; 2] = [0,0];
        codec.write(&buf)?;


        Ok(CodecControl {
            i2c_int: codec,
        })
    }
}