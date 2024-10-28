use std::thread::sleep;
use std::time::Duration;

use rppal::i2c::I2c;
use rppal::gpio::Gpio;
use crate::common::box_error::BoxError;




pub struct CodecControl {
    i2c_int: I2c,
}

impl CodecControl {
    pub fn new() -> Result<CodecControl, BoxError> {

        let mut codec = I2c::new()?;
        dbg!(&codec);
        codec.set_slave_address(0x18)?;

        // reset the code
        let mut pin = Gpio::new()?.get(17)?.into_output();
        pin.set_low();
        // Sleep for 200msec
        sleep(Duration::new(0, 200_000_000));
        pin.set_high();

        // Set Reg 0 - select Page 0
        let buf: [u8; 2] = [0,0];
        codec.write(&buf)?;


        Ok(CodecControl {
            i2c_int: codec,
        })
    }
}