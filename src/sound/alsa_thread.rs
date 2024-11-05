use alsa::direct::pcm::MmapPlayback;
use alsa::pcm::*;
use alsa::{Direction, ValueOr};

// use crate::JamEngine;
use crate::common::box_error::BoxError;

type SF = i16;
const FRAME_SIZE: usize = 128;

struct OutputBuffer {
    pos: usize,
    buf: [SF; FRAME_SIZE],
}

impl OutputBuffer {
    pub fn load(&mut self, buf: &[SF; FRAME_SIZE]) -> () {
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
            return Some(0);
        }
        let val = self.buf[self.pos];
        self.pos += 1;
        Some(val)
    }
}

fn start_capture(device: &str) -> Result<PCM, BoxError> {
    let pcm = PCM::new(device, Direction::Capture, false)?;
    {
        let hwp = HwParams::any(&pcm)?;
        hwp.set_channels(1)?;
        hwp.set_rate(48000, ValueOr::Nearest)?;
        hwp.set_format(Format::s16())?;
        hwp.set_access(Access::RWInterleaved)?;
        pcm.hw_params(&hwp)?;
    }
    pcm.start()?;
    Ok(pcm)
}

fn open_playback_dev(device: &str) -> Result<PCM, BoxError> {
    let req_samplerate = 48_000;
    let req_bufsize: i64 = (FRAME_SIZE * 2) as i64;  // A few ms latency by default, that should be nice

    // Open the device
    let p = alsa::PCM::new(device, alsa::Direction::Playback, false)?;

    // Set hardware parameters
    {
        let hwp = HwParams::any(&p)?;
        hwp.set_channels(1)?;
        hwp.set_rate(req_samplerate, alsa::ValueOr::Nearest)?;
        hwp.set_format(Format::s16())?;
        hwp.set_access(Access::MMapInterleaved)?;
        hwp.set_buffer_size(req_bufsize)?;
        hwp.set_period_size(req_bufsize, alsa::ValueOr::Nearest)?;
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

fn write_samples_io(p: &alsa::PCM, io: &mut alsa::pcm::IO<SF>, synth: &mut OutputBuffer) -> Result<bool, BoxError> {
    let avail = match p.avail_update() {
        Ok(n) => n,
        Err(e) => {
            println!("Recovering from {}", e);
            p.recover(e.errno() as std::os::raw::c_int, true)?;
            p.avail_update()?
        }
    } as usize;

    if avail > 0 {
        io.mmap(avail, |buf| {
            for sample in buf.iter_mut() {
                *sample = synth.next().unwrap()
            };
            buf.len() / 2
        })?;
    }
    use alsa::pcm::State;
    match p.state() {
        State::Running => Ok(false), // All fine
        State::Prepared => { println!("Starting audio output stream"); p.start()?; Ok(true) },
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
pub fn run(device: &str) -> Result<(), BoxError> {
    // Started by the client thread.  Our job is to read/write to the audio device
    let indev = start_capture(device)?;
    let io_in = indev.io_i16()?;
    let mut in_buf = [0; FRAME_SIZE];

    let outdev = open_playback_dev(device)?;
    // let mut mmap = outdev.direct_mmap_playback::<SF>()?;
    let mut io_out = outdev.io_i16()?;
    let mut out_buf = OutputBuffer {
        pos: 0,
        buf: [0; FRAME_SIZE],
    };

    // let io_out = outdev.io_i16()?;
    // let mut out_buf = [0i16, 128];
    loop {
        assert_eq!(io_in.readi(&mut in_buf)?, in_buf.len());
        // assert_eq!(io_out.writei(&in_buf)?, in_buf.len());
        // println!("rms = {}", rms(&in_buf));
        out_buf.load(&in_buf);

        // Write to output
        while write_samples_io(&outdev, &mut io_out, &mut out_buf)? { 
            // writing sample
            println!("wrote frame");
        }
    }

    // Ok(())
}
