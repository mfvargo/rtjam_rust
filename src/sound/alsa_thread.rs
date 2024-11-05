use alsa::direct::pcm::MmapPlayback;
use alsa::pcm::*;
use alsa::{Direction, ValueOr};

// use crate::JamEngine;
use crate::common::box_error::BoxError;
use crate::JamEngine;

type SF = i16;
const FRAME_SIZE: usize = 128;
const SAMPLE_RATE: u32 = 48_000;
const CHANNELS: u32 = 2;

struct OutputBuffer {
    pos: usize,
    buf: [SF; FRAME_SIZE * CHANNELS as usize],
}

impl OutputBuffer {
    pub fn new() -> OutputBuffer {
        OutputBuffer {
            pos: 0,
            buf: [0; FRAME_SIZE * CHANNELS as usize]
        }
    }
    pub fn load(&mut self, buf: &[SF]) -> () {
        let mut i: usize = 0;
        for v in buf {
            self.buf[i] = *v;
            i += 1;
        }
        self.pos = 0;
    }
}

impl Iterator for OutputBuffer {
    type Item = SF;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= FRAME_SIZE {
            return None;
        }
        let val = self.buf[self.pos];
        self.pos += 1;
        Some(val)
    }
}

fn open_record_dev(device: &str) -> Result<PCM, BoxError> {
    let pcm = PCM::new(device, Direction::Capture, false)?;
    {
        let hwp = HwParams::any(&pcm)?;
        hwp.set_channels(CHANNELS)?;
        hwp.set_rate(SAMPLE_RATE, ValueOr::Nearest)?;
        hwp.set_format(Format::s16())?;
        hwp.set_access(Access::RWInterleaved)?;
        pcm.hw_params(&hwp)?;
    }
    Ok(pcm)
}

fn open_playback_dev(device: &str) -> Result<PCM, BoxError> {
    let req_bufsize: i64 = (FRAME_SIZE * 2) as i64;  // A few ms latency by default, that should be nice

    // Open the device
    let p = alsa::PCM::new(device, alsa::Direction::Playback, false)?;

    // Set hardware parameters
    {
        let hwp = HwParams::any(&p)?;
        hwp.set_channels(CHANNELS)?;
        hwp.set_rate(SAMPLE_RATE, alsa::ValueOr::Nearest)?;
        hwp.set_format(Format::s16())?;
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
        println!("Opened audio output {:?} with parameters: {:?}, {:?}", device, hwp, swp);
        hwp.get_rate()?
    };

    Ok(p)
}


fn write_samples_direct(p: &alsa::PCM, mmap: &mut MmapPlayback<SF>, outbuf: &mut OutputBuffer)
    -> Result<bool, BoxError> {

    if mmap.avail() > 0 {
        // Write samples to DMA area from iterator
        mmap.write(outbuf);
    }
    use alsa::pcm::State;
    match mmap.status().state() {
        State::Running => { return Ok(false); }, // All fine
        State::Prepared => { println!("Starting audio output stream"); p.start()? },
        State::XRun => { println!("Underrun in audio output stream!"); p.prepare()? },
        State::Suspended => { println!("Resuming audio output stream"); p.resume()? },
        n @ _ => Err(format!("Unexpected pcm state {:?}", n))?,
    }
    Ok(true) // Call us again, please, there might be more data to write
}

fn write_samples_io(p: &alsa::PCM, io: &mut alsa::pcm::IO<SF>, buf: &mut OutputBuffer) -> Result<bool, BoxError> {
    let avail = match p.avail_update() {
        Ok(n) => n,
        Err(e) => {
            println!("Recovering from {}", e);
            p.recover(e.errno() as std::os::raw::c_int, true)?;
            p.avail_update()?
        }
    } as usize;

    println!("PRE: avail: {}, state: {:?}", avail, p.state());

    if avail >= FRAME_SIZE {
        let frames = io.mmap(avail, |b| {
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
                // *sample = buf.next().unwrap()
            };
            count / CHANNELS as usize
        })?;
        println!("Wrote {} franes", frames);
    }
    use alsa::pcm::State;
    match p.state() {
        State::Running => Ok(false), // All fine
        State::Prepared => { println!("Starting audio output stream"); p.start().unwrap(); Ok(true) },
        State::Suspended | State::XRun => Ok(true), // Recover from this in next round
        n @ _ => Err(format!("Unexpected pcm state {:?}", n))?,
    }
}

// Calculates RMS (root mean square) as a way to determine volume
fn rms(buf: &[i16]) -> f64 {
    if buf.len() == 0 { return 0f64; }
    let mut sum = 0f64;
    for &x in buf {
        sum += (x as f64) * (x as f64);
    }
    let r = (sum / (buf.len() as f64)).sqrt();
    // Convert value to decibels
    20.0 * (r / (i16::MAX as f64)).log10()
}

// Run the loop to read/write alsa
pub fn run(mut engine: JamEngine) -> Result<(), BoxError> {
    let device = "hw:CODEC";
    // Started by the client thread.  Our job is to read/write to the audio device
    let indev = open_record_dev(device)?;
    indev.start()?;
    let io_in = indev.io_i16()?;
    let mut in_buf = [0; FRAME_SIZE * CHANNELS as usize];

    let outdev = open_playback_dev(device)?;
    // let mut mmap = outdev.direct_mmap_playback::<SF>()?;
    let mut io_out = outdev.io_i16()?;
    // outdev.start().unwrap();
    let mut out_buf = OutputBuffer::new();

    // let io_out = outdev.io_i16()?;
    // let mut out_buf = [0i16, 128];
    let mut in_a: [f32; FRAME_SIZE] = [0.0; FRAME_SIZE];
    let mut in_b: [f32; FRAME_SIZE] = [0.0; FRAME_SIZE];
    loop {
        assert_eq!(io_in.readi(&mut in_buf)?, FRAME_SIZE);

        //  Convert the input date into f32 for the engine:
        let mut i = 0;
        for v in in_buf {
            if i%2 == 0 {
                in_a[i/2] = v as f32;
            } else {
                in_b[i/2] = v as f32;
            }
            i += 1;
        }
        engine.process_inputs(&in_a, &in_b);
        // println!("rms = {}", rms(&in_buf));
        out_buf.load(&in_buf);

        // Write to output
        while write_samples_io(&outdev, &mut io_out, &mut out_buf)? { 
            // writing sample
            // println!("wrote frame");
        }
    }

    // Ok(())
}
