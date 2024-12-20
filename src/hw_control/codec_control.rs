use log::debug;

use std::fmt;
use std::thread::sleep;
use std::time::Duration;

use rppal::i2c::I2c;
use rppal::gpio::Gpio;
use crate::common::box_error::BoxError;
use pedal_board::dsp::smoothing_filter::SmoothingFilter;

// Helper functions to encode and decode binary-coded decimal (BCD) values.
pub fn bcd2dec(bcd: u8) -> u8 {
    (((bcd & 0xF0) >> 4) * 10) + (bcd & 0x0F)
}

pub fn dec2bcd(dec: u8) -> u8 {
    ((dec / 10) << 4) | (dec % 10)
}


pub struct CodecControl {
    i2c_int: I2c,
    adc_values: [u16; 4],
    pot_values: [u8; 3],
    prev_pot_values: [u8; 3],
    filters: [SmoothingFilter; 3],
}

impl fmt::Display for CodecControl {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "adc: {}, {}, {} pot: {:.2}, {:.2}, {:.2}",
            self.adc_values[0],
            self.adc_values[1],
            self.adc_values[2],
            self.pot_values[0],
            self.pot_values[1],
            self.pot_values[2],
        )
    }
}


// function to test if pot values have changed (handles wrapping better than old method)
pub fn pot_has_changed(old_value: u8, new_value: u8) -> bool {
        
    // Calculate the difference using wrapping arithmetic
    let diff = old_value.wrapping_sub(new_value).abs_diff(0);
    
    // Check if the difference is 2 or more
    diff > 2
}

impl CodecControl {
    pub fn new() -> Result<CodecControl, BoxError> {

        let mut codec = I2c::new()?;
        dbg!(&codec);

        // reset the code
        let mut pin = Gpio::new()?.get(17)?.into_output();
        pin.set_high();
        // Sleep for 200msec
        sleep(Duration::new(0, 200_000_000));
        pin.set_low();

        // Let the thing come up
        sleep(Duration::new(0, 200_000_000));

        codec.set_slave_address(0x18)?;

        // Set Reg 0 - select Page 0
        let mut buf: [u8; 2] = [0,0];
        codec.write(&buf)?;

        let reg_data_p0: [[u8;2]; 20] = [
            [07, 0x0A], // Reg 7 - 48ksps, single rate, L and R path enables
            [08, 0xC0], // Reg 8 - Master Mode - bclk and FS are outputs
            [09, 0x30], // Reg 9 - I2S mode, 32 bit, no re-sync
            [14, 0x80], // Reg 14 - configure headphone output driver for ac coupled mode
            [18, 0x0F], // Reg 18 - mic2l pin in to right ADC channel input
            [19, 0x04], // Reg 19 - power up ADC left channel
            [22, 0x04], // Reg 22 - power up ADC right channel
            [25, 0x80], // Reg 25 - headset mic bias out = 2.5V
            [37, 0xC0], // Reg 37 - power up left and right DAC
            [43, 0x00], // Reg 43 - check
            [44, 0x00], // Reg 44 - check 
            [47, 0x80], // Reg 47 - route DAC L1 to HPLOUT pin
            [51, 0x0F], // Reg 51 - unmoute and power up HPLOUT
            [64, 0x80], // Reg 64 - route DAC R1 to HPROUT pin
            [65, 0x0F], // Reg 65 - HPLCOM powered up
            [82, 0x80], // Reg 82 - Route L DAC To LEFT_LOP pin 
            [86, 0x09], // Reg 86 - Power up LEFT_LOP 
            [92, 0x80], // Reg 92 - Route R DAC To RIGHT_LOP pin
            [93, 0x09], // Reg 93 - Power up RIGHT_LOP 
            [101, 0x01] // Reg 101 - CODEC_CLKIN uses CLKDIV_OUT
        ];

        // Init Page 0 Registers
        for pair in reg_data_p0 {
            buf[0] = pair[0];
            buf[1] = pair[1];
            codec.write(&buf)?;
        }

        // Coefficients for Codec internal hardware filter - implements a 10Hz DC-Blocking filter    
        // on left and right ADC channel inputs (see page 81 of datasheet)
        let reg_data_p1: [[u8;2]; 12] = [
            [65, 0x7F], [66, 0xE9], [67, 0x80], [68, 0x17], [69, 0x7F], [70, 0xD4], 
            [71, 0x7F], [72, 0xE9], [73, 0x80], [74, 0x17], [75, 0x7F], [76, 0xD4]
        ];

        // Set Reg 1 - select Page 1
        buf = [0,1];
        codec.write(&buf)?;

        // Init ADC HPF filter coeffs/enable 10Hz filter to remove DC offset 
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
            pot_values: [0, 0, 0],
            prev_pot_values: [0, 0, 0],
            filters: [
                SmoothingFilter::build(0.1, 200.0),
                SmoothingFilter::build(0.1, 200.0),
                SmoothingFilter::build(0.1, 200.0),
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
            self.pot_values[i] = self.filters[i].get(self.adc_values[i] as f64) as u8;
          //  debug!("Pot {} Value = {:#02x}", i, self.pot_values[i] as u8);
        }

        // Setup i2c bus to talk to the codec
        self.i2c_int.set_slave_address(0x18)?;

        // Initialize Buffer to be used to write to i2c  devices
        let mut buf: [u8; 2] = [0,0];

        // Pot 1 - channel 0 - Instrument input gain
        // Codec Register 15 (0x0F) - Left-ADC PGA Gain Control Register
        // Range 0x00-0x7f (bit 7 is mute)
        // Gain = 0.5dB per step
        // instrument input 1 limit to max gain of 255/5*(0.5) = 25.5dB max for guitar and bass 
        if pot_has_changed(self.prev_pot_values[0], self.pot_values[0]) {    
            buf[0] = 15;
            buf[1] = (self.pot_values[0] / 5).clamp(0, 255) as u8;
            self.i2c_int.write(&buf)?;
            self.prev_pot_values[0] = self.pot_values[0];
            debug!("Pot 1 Changed - Wrote {:#02x} to Codec register {:#02x}", buf[1], buf[0]);
        }

        // Pot 2 - channel 1 - mic/headset input gain
        // Codec Register 16 (0x10) - Right-ADC PGA Gain Control Register
        // Range 0x00-0x7f (bit 7 is mute)
        // Gain = 0.5dB per step
        // mic input input max gain = 255/4*(0.5) = 31.5dB max for guitar and bass 
        // total gain of mic input = gain of external input diff-amp + internal PGA 
        // actual input gain = +20dB to + 51.5dB
        if pot_has_changed(self.prev_pot_values[1], self.pot_values[1]) {
            buf[0] = 16;
            buf[1] = (self.pot_values[1] / 4).clamp(0, 255) as u8;
            self.i2c_int.write(&buf)?;
            self.prev_pot_values[1] = self.pot_values[1];
            debug!("Pot 2 Changed - Wrote {:#02x} to Codec register {:#02x}", buf[1], buf[0]);
        }

        // Pot 3 - channel 2 - Headphone amp gain 
        // Code Register 47 (0x2F) - DAC_L1 to HPLOUT Volume Control Register
        // Range of attenuation from full-scale = -0.5dB per step
        // max attenuation of headphone out = 127/2*(-0.5) = -63.5db - not off but very quiet...
        // write to both registers 47 (DAC_L1->headphone vol) and 64 (right DAC_R1 headphone vol)
        if pot_has_changed(self.prev_pot_values[2], self.pot_values[2]) {
            buf[0] = 47;   
            buf[1] = 255 - ((self.pot_values[2]/2).clamp(0, 255)) as u8;
            buf[1] |= 0x80; // set bit 7 before right - routes DAC output to HP out
            self.i2c_int.write(&buf)?;
            buf[0] = 64;    
            self.i2c_int.write(&buf)?;
            self.prev_pot_values[2] = self.pot_values[2];
            debug!("Pot 3 Changed - Wrote {:#02x} to Codec register {:#02x}", buf[1], buf[0]);
        }

        Ok(())
    }

    fn adc_scan_inputs(&mut self) -> Result<(), BoxError> {

        // setup i1c to write to ADC at address 0x29
        self.i2c_int.set_slave_address(0x29)?;
        let start_conv: [u8; 1] = [0xf0];
        assert!(self.i2c_int.write(&start_conv)? == 1);
        sleep(Duration::new(0, 10_000));    // 10us delay 

        let mut buf: [u8; 2] = [0,0];

        // read all 4 channels of ADC - ADC will increment the channel after each read.
        // the 4th value is noise - not connected to anything on the board.
        for _i in [0, 1, 2, 3] {
            self.i2c_int.read(&mut buf)?;   // Read data
            let adc_chan = ((buf[0] & 0x30) >> 4) as usize;  // Get channel ID as usize for indexing
            // Extract the full 12-bit ADC value by including all 8 bits from buf[0] and buf[1]
            let value = (((buf[0] & 0x0F) as u16) << 8) | ((buf[1] & 0xFF) as u16);  //  lower nibble of buf[0] =MSB buf[1] = LSB           
            self.adc_values[adc_chan] = value/16;  // scale and copy final 8 bit ADC value to array of ADC values
          //  debug!("ADC ch: {}, buf[0]: {:#02x}, buf[1]: {:#02x}, adc_val: {:#04x}", adc_chan, buf[0], buf[1], self.adc_values[adc_chan]);
        }

        Ok(())
    }
}