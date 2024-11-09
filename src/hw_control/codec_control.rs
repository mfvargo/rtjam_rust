use std::thread::sleep;
use std::time::Duration;

use rppal::i2c::I2c;
use rppal::gpio::Gpio;
use crate::common::box_error::BoxError;
use crate::dsp::smoothing_filter::SmoothingFilter;

// Helper functions to encode and decode binary-coded decimal (BCD) values.
pub fn bcd2dec(bcd: u8) -> u8 {
    (((bcd & 0xF0) >> 4) * 10) + (bcd & 0x0F)
}

pub fn dec2bcd(dec: u8) -> u8 {
    ((dec / 10) << 4) | (dec % 10)
}


pub struct CodecControl {
    i2c_int: I2c,
    adc_values: [u64; 4],
    pot_values: [f64; 3],
    prev_pot_values: [f64; 3],
    filters: [SmoothingFilter; 3],
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
        let mut buf: [u8; 2] = [0,0];
        codec.write(&buf)?;

        let reg_data_p0: [[u8;2]; 20] = [
            [07, 0x0A], [08, 0xC0], [09, 0x30], [14, 0x80], [18, 0x0F], [19, 0x04], [22, 0x04], 
            [25, 0x80], [37, 0xC0], [43, 0x00], [44, 0x00], [47, 0x80], [51, 0x0F], [64, 0x80], 
            [65, 0x0F], [82, 0x80], [86, 0x09], [92, 0x80], [93, 0x09], [101, 0x01]
        ];

        // Init Page 0 Registers
        for pair in reg_data_p0 {
            buf[0] = pair[0];
            buf[1] = pair[1];
            codec.write(&buf)?;
        }

        let reg_data_p1: [[u8;2]; 12] = [
            [65, 0x7F], [66, 0xE9], [67, 0x80], [68, 0x17], [69, 0x7F], [70, 0xD4], 
            [71, 0x7F], [72, 0xE9], [73, 0x80], [74, 0x17], [75, 0x7F], [76, 0xD4]
        ];

        // Set Reg 1 - select Page 1
        buf = [0,1];
        codec.write(&buf)?;

        // Init ADC HPF filter coeffs - 10Hz filter to remove DC offset
        for pair in reg_data_p1 {
            buf[0] = pair[0];
            buf[1] = pair[1];
            codec.write(&buf)?;
        }
        
        // Set Reg 0 - select Page 0
        buf = [0,0];
        codec.write(&buf)?;

        // Set Reg 12 - Enable Left and Right ADC Channel HPF
        buf = [12,0x50];
        codec.write(&buf)?;

        // Set Reg 107 - set HPF to use custom coeffs loaded above
        buf = [107,0xc0];
        codec.write(&buf)?;

        Ok(CodecControl {
            i2c_int: codec,
            adc_values: [0, 0, 0, 0],
            pot_values: [0.0, 0.0, 0.0],
            prev_pot_values: [0.0, 0.0, 0.0],
            filters: [
                SmoothingFilter::build(1.0, 48_000.0),
                SmoothingFilter::build(1.0, 48_000.0),
                SmoothingFilter::build(1.0, 48_000.0),
            ]
        })
    }

    // This function will catch errors so the hw_control_thread does not die if the pot reads go bad
    pub fn read_pots(&mut self) -> () {
        // This is where we would put code to poll the pots and set registers on the coded...
        match self.update_volumes() {
            Ok(()) => {
                // No errors reading the i/o and updating stuff
            }
            Err(e) => {
                // Error reading the pots
                dbg!(e);
            }
        }
    }
    
    fn update_volumes(&mut self) -> Result<(), BoxError> {
        self.adc_scan_inputs()?;

        // Filter the values
        for i in [0, 1, 2] {
            self.pot_values[i] = self.filters[i].get(self.adc_values[i] as f64);
        }

        // Setup i2c bus to talk to the codec
        self.i2c_int.set_slave_address(0x18)?;

        // Initialize Buffer to be used to write to i2c  devices
        let mut buf: [u8; 2] = [0,0];

        // Pot 1 - channel 0 - Instrument input gain
        if f64::abs(self.prev_pot_values[0] - self.pot_values[0]) > 2.0 {
            buf[0] = 15;
            buf[1] = (self.pot_values[0] / 5.0).clamp(0.0, 255.0) as u8;
            self.i2c_int.write(&buf)?;
            self.prev_pot_values[0] = self.pot_values[0];
        }

        // Pot 2 - channel 1 - mic/headset input gain
        if f64::abs(self.prev_pot_values[1] - self.pot_values[1]) > 2.0 {
            buf[0] = 16;
            buf[1] = (self.pot_values[1] / 4.0).clamp(0.0, 255.0) as u8;
            self.i2c_int.write(&buf)?;
            self.prev_pot_values[1] = self.pot_values[1];
        }

        // Pot 3 - channel 2 - Headphone amp gain
        if f64::abs(self.prev_pot_values[2] - self.pot_values[2]) > 2.0 {
            buf[0] = 47;
            buf[1] = 255 - ((self.pot_values[2]/2.0).clamp(-127.99, 127.99)) as u8;
            buf[1] |= 0x80;
            self.i2c_int.write(&buf)?;
            self.prev_pot_values[2] = self.pot_values[2];
        }

        Ok(())
    }

    fn adc_scan_inputs(&mut self) -> Result<(), BoxError> {

        // setup i1c to write to ADC at address 0x29
        self.i2c_int.set_slave_address(0x29)?;

        let start_conv: [u8; 1] = [0xf0];
        self.i2c_int.write(&start_conv)?;

        sleep(Duration::new(0, 10_000));

        let mut buf: [u8; 2] = [0,0];

        // read all 4 channels of ADC - ADC will increment the channel after each read.
        // the 4th value is noise - not connected to anything on the board.
        for _i in [0, 1, 2, 4] {
            self.i2c_int.read(&mut buf)?;   // Read data
            let adc_chan = ((buf[0] & 0x30) >> 4) as usize;  // Get channel ID as usize for indexing
            // Extract the full 12-bit ADC value by shifting and ORing buf[0] and buf[1]
            let value = (((buf[0] & 0x0F) as u64) << 4) | ((buf[1] >> 4) as u64);  // Combine full byte of buf[0] and buf[1]
            self.adc_values[adc_chan] = value;  // Scale down for copy ADC value to array of ADC values
        }

        Ok(())
    }
}