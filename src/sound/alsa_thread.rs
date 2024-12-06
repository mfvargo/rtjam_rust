// use alsa::direct::pcm::MmapPlayback;
use alsa::pcm::*;
use alsa::{Direction, ValueOr};

// use crate::JamEngine;
use crate::common::box_error::BoxError;
use crate::common::get_micro_time;
use crate::common::stream_time_stat::{MicroTimer, StreamTimeStat};
use crate::JamEngine;

use log::{trace, debug, /*info,*/ warn, error};

type SF = i16;
const FRAME_SIZE: usize = 128;
const SAMPLE_RATE: u32 = 48_000;
const CHANNELS: u32 = 2;
const MAX_SAMPLE: f32 = 32766.0;
const SMP_FORMAT: Format = Format::s16();

// Iterable output buffer that converts from floats to ints for alsa device
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
        if self.pos >= FRAME_SIZE * CHANNELS as usize {
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
        hwp.set_format(SMP_FORMAT)?;
        hwp.set_access(Access::RWInterleaved)?;
        hwp.set_buffer_size(2 * FRAME_SIZE as i64)?;
        hwp.set_period_size(FRAME_SIZE as i64, alsa::ValueOr::Nearest)?;
        pcm.hw_params(&hwp)?;
    }
    debug!("::open_record_dev - Opened audio input with parameters: {:?}, {:?}", pcm.hw_params_current(), pcm.sw_params_current());
    Ok(pcm)
}

fn open_playback_dev(device: &str) -> Result<PCM, BoxError> {
    let req_bufsize: i64 = (FRAME_SIZE * 4) as i64;  // A few ms latency by default, that should be nice

    // Open the device
    let p = alsa::PCM::new(device, alsa::Direction::Playback, false)?;

    // Set hardware parameters
    {
        let hwp = HwParams::any(&p)?;
        hwp.set_channels(CHANNELS)?;
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
        debug!("::open_playback_dev - Opened audio output {:?} with parameters: {:?}, {:?}", device, hwp, swp);
        hwp.get_rate()?
    };

    Ok(p)
}


// fn write_samples_direct(p: &alsa::PCM, mmap: &mut MmapPlayback<SF>, outbuf: &mut OutputBuffer)
//     -> Result<bool, BoxError> {

//     if mmap.avail() > 0 {
//         // Write samples to DMA area from iterator
//         mmap.write(outbuf);
//     }
//     use alsa::pcm::State;
//     match mmap.status().state() {
//         State::Running => { return Ok(false); }, // All fine
//         State::Prepared => { println!("Starting audio output stream"); p.start()? },
//         State::XRun => { println!("Underrun in audio output stream!"); p.prepare()? },
//         State::Suspended => { println!("Resuming audio output stream"); p.resume()? },
//         n @ _ => Err(format!("Unexpected pcm state {:?}", n))?,
//     }
//     Ok(true) // Call us again, please, there might be more data to write
// }

fn write_samples_io(p: &alsa::PCM, io: &mut alsa::pcm::IO<SF>, buf: &mut OutputBuffer) -> Result<bool, BoxError> {
    let avail = match p.avail_update() {
        Ok(n) => n,
        Err(e) => {
            debug!("::write_samples_io - avail_update error {}. Attempting recover.", e);
            p.recover(e.errno() as std::os::raw::c_int, true)?;
            p.avail_update()?
        }
    } as usize;

    // write the data to the alsa device when there is room
    trace!("::write_samples_io - available frames: {}", avail);
    if avail >= FRAME_SIZE {
        trace!("::write_samples_io - begin mapping ...");
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
            trace!("::write_samples_io - frames mapped: {}", count);
            count / CHANNELS as usize
        })?;
    }
    use alsa::pcm::State;
    trace!("::write_samples_io - pcm state {:?}", p.state());
    match p.state() {
        State::Running => Ok(false), // All fine
        State::Prepared => { trace!("::write_samples_io - Starting audio output stream"); p.start()?; Ok(true) },
        State::Suspended | State::XRun => Ok(true), // Recover from this in next round
        n @ _ => { debug!("::write_samples_io - Unexpected pcm state"); Err(format!("Unexpected pcm state {:?}", n))? },
    }
}

// Run the loop to read/write alsa
pub fn run(mut engine: JamEngine, in_device: &str, out_device: &str) -> Result<(), BoxError> {
    // stats for callback
    let mut stats = StreamTimeStat::new(100);
    let mut timer = MicroTimer::new(get_micro_time(), 10_000);
    let mut frame_count: usize = 0;

    let indev = match open_record_dev(in_device) {
        Ok(dev) => {
            dev.start()?;
            dev
        }
        Err(e) => {
            error!("::run - Failed to open input device: {}", e);
            return Err(e);
        }
    };
    let io_in = indev.io_i16()?;
    let mut in_buf = [0; FRAME_SIZE * CHANNELS as usize];

    let outdev = open_playback_dev(out_device)?;
    // let mut mmap = outdev.direct_mmap_playback::<SF>()?;
    let mut io_out = outdev.io_i16()?;
    let mut out_buf = OutputBuffer::new();

    // Buffers for processing in f32
    let mut in_a: [f32; FRAME_SIZE] = [0.0; FRAME_SIZE];
    let mut in_b: [f32; FRAME_SIZE] = [0.0; FRAME_SIZE];
    let mut out_a: [f32; FRAME_SIZE] = [0.0; FRAME_SIZE];
    let mut out_b: [f32; FRAME_SIZE] = [0.0; FRAME_SIZE];

    while engine.is_running() {
        frame_count += 1;
        let now = get_micro_time();

        match io_in.readi(&mut in_buf) {
            Ok(samps) => {
                if samps < FRAME_SIZE {
                    warn!("::run - IO read samples: {}, below {} FRAME_SIZE", samps, FRAME_SIZE);
                    continue;
                }
            }
            Err(e) => {
                error!("::run - IO read failure: {}", e);
                indev.recover(e.errno() as std::os::raw::c_int, true)?;
            }
        }

        // stats on read i/o jitter
        stats.add_sample(timer.since(now) as f64);
        timer.reset(now);
        if frame_count%1000 == 0 {
            debug!("::run - stats: {}", stats);
        }

        //  Convert the input date from interleaved i16 into f32 for the engine:
        let mut i = 0;
        for v in in_buf {
            if i%2 == 0 {
                in_a[i/2] = v as f32 / MAX_SAMPLE;
            } else {
                in_b[i/2] = v as f32 / MAX_SAMPLE;
            }
            i += 1;
        }
        engine.process_inputs(&in_a, &in_b);


        // Here we write until the outdev does not have space for a frame
        let mut avail = FRAME_SIZE;  // set avail on the first time so it will at least try
        while avail >= FRAME_SIZE {
            // Now figure out how much we need to feed the output
            avail = match outdev.avail_update() {
                Ok(n) => n,
                Err(e) => {
                    warn!("::run - Error from avail_update: {}. Recovering.", e);
                    outdev.recover(e.errno() as std::os::raw::c_int, true)?;
                    outdev.avail_update()?
                }
            } as usize;

            if avail >= FRAME_SIZE {
                // We need to feed the meter
                engine.get_playback_data(&mut out_a, &mut out_b);
                out_buf.load_data(&out_a, &out_b);
                // out_buf.load(&in_buf);
                // out_buf.load(&in_buf);
                // lets try to write a frame to the io device
                // Might have to recurse in there based on state, hence the pumping
                let mut pumping = true;
                while pumping {
                    match write_samples_io(&outdev, &mut io_out, &mut out_buf) {
                        Ok(more) => {
                            pumping = more;
                        }
                        Err(e) => {
                            pumping = false;
                            debug!("::run - Error collecting samples: {e}");
                        }
                    }
                }
                avail -= FRAME_SIZE;
            }
        }
    }

    Ok(())
}
