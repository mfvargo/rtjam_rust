/// The alsa-device encapsulates the i/o setup for an full duplex stereo low latency audio
/// stream .  To use the AlsaDevice, you ::new() one giving the names of the input and output
/// devices.  The device will that expect to have its process_a_frame called repeatedly.  This 
/// will take a &mut Callback as an argument.  Any struct that implements the Callback trace will then
/// have their call function implementation called with the input and output buffers for the 
/// alsa device (in f32 format).  Very much like the jack callback function.
use std::fmt;
use alsa::pcm::*;
use alsa::{Direction, ValueOr};
use log::{error, info};

use crate::common::box_error::BoxError;
use super::SoundCallback;


type SF = i16;
pub const FRAME_SIZE: usize = 128;
const SAMPLE_RATE: u32 = 48_000;
pub const CHANNELS: usize = 2;
const MAX_SAMPLE: f32 = 32766.0;
const SMP_FORMAT: Format = Format::s16();

struct OutputBuffer {
    pos: usize,
    buf: [SF; FRAME_SIZE * CHANNELS],
}

impl OutputBuffer {
    pub fn new() -> OutputBuffer {
        OutputBuffer {
            pos: 0,
            buf: [0; FRAME_SIZE * CHANNELS]
        }
    }
    pub fn load_data(&mut self, out_a: &[f32], out_b: &[f32]) -> () {
        // interleave and convert floats
        let mut i: usize = 0;
        while i < FRAME_SIZE {
            self.buf[i*2] = (out_a[i] * MAX_SAMPLE) as i16;
            self.buf[i*2+1] = (out_b[i] * MAX_SAMPLE) as i16;
            i += 1;
        }
        self.pos = 0;
    }
}

impl Iterator for OutputBuffer {
    type Item = SF;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= FRAME_SIZE * CHANNELS  {
            return None;
        }
        let val = self.buf[self.pos];
        self.pos += 1;
        Some(val)
    }
}

pub struct AlsaDevice {
    in_dev: PCM,
    out_dev: PCM,
    out_buf: OutputBuffer,
}

impl fmt::Display for AlsaDevice {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "in_dev: {:?}, out_dev: {:?}",
            self.in_dev.state(), 
            self.out_dev.state(),
        )
    }
}

impl AlsaDevice{
    // Create a new full duplex alsa device
    pub fn new(input: &str, output: &str) -> Result<AlsaDevice, BoxError> {
        let device = AlsaDevice {
            in_dev: open_record_dev(input)?,
            out_dev: open_playback_dev(output)?,
            out_buf: OutputBuffer::new(),
        };
        device.in_dev.start()?;
        Ok(device)
    }
    // function to loop reading data etc
    pub fn run(&mut self, engine: &mut dyn SoundCallback) -> Result<(), BoxError> {
        let mut io_out = self.out_dev.io_i16()?;
        let io_in = self.in_dev.io_i16()?;
        let mut in_buf = [0; FRAME_SIZE * CHANNELS];
        let mut out_buf = OutputBuffer::new();


        // Buffers for processing in f32
        let mut in_a: [f32; FRAME_SIZE] = [0.0; FRAME_SIZE];
        let mut in_b: [f32; FRAME_SIZE] = [0.0; FRAME_SIZE];
        let mut out_a: [f32; FRAME_SIZE] = [0.0; FRAME_SIZE];
        let mut out_b: [f32; FRAME_SIZE] = [0.0; FRAME_SIZE];
        
        while engine.is_running() {
            // Read in a frame
            match io_in.readi(&mut in_buf) {
                Ok(samps) => {
                    if samps < FRAME_SIZE {
                        error!("Not enough samples: {}", samps);
                        continue;
                    }
                }
                Err(e) => {
                    error!("read error: {}", dbg!(e));
                    self.in_dev.recover(e.errno() as std::os::raw::c_int, true)?;
                }
            }
            //  Convert the input date from interleaved i16 into f32 for the pedalboard:
            for (i, v) in in_buf.iter().enumerate() {
                if i%2 == 0 {
                    in_a[i/2] = *v as f32 / MAX_SAMPLE;
                } else {
                    in_b[i/2] = *v as f32 / MAX_SAMPLE;
                }
            }
            // Process the inputs
            engine.process_inputs(&in_a, &in_b);

            // Here we write until the outdev does not have space for a frame
            let mut avail = match self.out_dev.avail_update() {
                Ok(n) => n,
                Err(e) => {
                    error!("Recovering from {}", e);
                    self.out_dev.recover(e.errno() as std::os::raw::c_int, true)?;
                    self.out_dev.avail_update()?
                }
            } as usize;
            while avail >= FRAME_SIZE {
                if avail >= FRAME_SIZE {
                    // We need to feed the meter
                    engine.get_playback_data(&mut out_a, &mut out_b);
                    out_buf.load_data(&out_a, &out_b);
                    // lets try to write a frame to the io device
                    // Might have to recurse in there based on state, hence the pumping
                    let mut pumping = true;
                    while pumping {
                        match write_samples_io(&self.out_dev, &mut io_out, &mut self.out_buf) {
                            Ok(more) => {
                                // debug!("pumping: {}", more);
                                pumping = more;
                            }
                            Err(e) => {
                                pumping = false;
                                error!("Error writing to device: {}", e);
                            }
                        }
                    }
                    avail -= FRAME_SIZE;
                }
            }        
        }
        Ok(())
    }
}

fn open_record_dev(device: &str) -> Result<PCM, BoxError> {
    let pcm = PCM::new(device, Direction::Capture, false)?;
    {
        let hwp = HwParams::any(&pcm)?;
        hwp.set_channels(CHANNELS as u32)?;
        hwp.set_rate(SAMPLE_RATE, ValueOr::Nearest)?;
        hwp.set_format(SMP_FORMAT)?;
        hwp.set_access(Access::RWInterleaved)?;
        hwp.set_buffer_size(2 * FRAME_SIZE as i64)?;
        hwp.set_period_size(FRAME_SIZE as i64, alsa::ValueOr::Nearest)?;
        pcm.hw_params(&hwp)?;
    }
    info!("Opened audio input with parameters: {:?}, {:?}", pcm.hw_params_current(), pcm.sw_params_current());
    Ok(pcm)
}

fn open_playback_dev(device: &str) -> Result<PCM, BoxError> {
    let req_bufsize: i64 = (FRAME_SIZE * 4) as i64;  // A few ms latency by default, that should be nice

    // Open the device
    let p = alsa::PCM::new(device, alsa::Direction::Playback, false)?;

    // Set hardware parameters
    {
        let hwp = HwParams::any(&p)?;
        hwp.set_channels(CHANNELS as u32)?;
        hwp.set_rate(SAMPLE_RATE, alsa::ValueOr::Nearest)?;
        hwp.set_format(SMP_FORMAT)?;
        hwp.set_access(Access::MMapInterleaved)?;
        hwp.set_buffer_size(req_bufsize)?;
        hwp.set_period_size(req_bufsize / 4, alsa::ValueOr::Nearest)?;
        p.hw_params(&hwp)?;
    }

    // Set software parameters
    let _rate = {
        let hwp = p.hw_params_current()?;
        let swp = p.sw_params_current()?;
        let (bufsize, periodsize) = (hwp.get_buffer_size()?, hwp.get_period_size()?);
        swp.set_start_threshold(bufsize - periodsize)?;
        swp.set_avail_min(periodsize)?;
        p.sw_params(&swp)?;
        info!("Opened audio output {:?} with parameters: {:?}, {:?}", device, hwp, swp);
        hwp.get_rate()?
    };

    Ok(p)
}

fn write_samples_io(p: &alsa::PCM, io: &mut alsa::pcm::IO<SF>, buf: &mut OutputBuffer) -> Result<bool, BoxError> {
    let avail = match p.avail_update() {
        Ok(n) => n,
        Err(e) => {
            info!("Recovering from {}", e);
            p.recover(e.errno() as std::os::raw::c_int, true)?;
            p.avail_update()?
        }
    } as usize;

    // write the data to the alsa device when there is room
    if avail >= FRAME_SIZE {
        io.mmap(FRAME_SIZE, |b| {
            let mut count = 0;
            for sample in b.iter_mut() {
                match buf.next() {
                    Some(v) => {
                        count += 1;
                        *sample = v
                    }
                    None => {
                        break
                    }
                }
            };
            count / CHANNELS as usize
        })?;
    }
    use alsa::pcm::State;
    match p.state() {
        State::Running => Ok(false), // All fine
        State::Prepared => { info!("Starting audio output stream"); p.start()?; Ok(true) },
        State::Suspended | State::XRun => Ok(true), // Recover from this in next round
        n @ _ => Err(format!("Unexpected pcm state {:?}", n))?,
    }
}