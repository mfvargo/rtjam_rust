use std::thread::JoinHandle;

use alsa::pcm::*;
use alsa::{Direction, ValueOr};

#[cfg(test)]
use mockall::automock;

// use crate::JamEngine;
use crate::common::box_error::BoxError;
use crate::common::stream_time_stat::{/*MicroTimer,*/ StreamTimeStat};
//use crate::JamEngine;

use log::{ debug,/* trace, info, warn, error */};

//type SF = i16;
const FRAME_SIZE: usize = 128;
const SAMPLE_RATE: u32 = 48_000;
const CHANNELS: u32 = 2;
//const MAX_SAMPLE: f32 = 32766.0;
const SMP_FORMAT: Format = Format::s16();


#[cfg_attr(test, automock)]
pub trait InputDevice: Send + Sync {
    fn read(&mut self, buffer: &mut [i16]) -> Result<usize, BoxError>;
    fn start(&mut self) -> Result<(), BoxError>;
    fn recover(&mut self, errno: i32) -> Result<(), BoxError>;
}

#[cfg_attr(test, automock)]
pub trait OutputDevice: Send + Sync {
    fn write(&mut self, buffer: &[i16]) -> Result<usize, BoxError>;
    fn avail_update(&self) -> Result<i64, BoxError>;
    fn recover(&mut self, errno: i32) -> Result<(), BoxError>;
}

// Add wrapper struct
struct ThreadSafePCM(PCM);
unsafe impl Send for ThreadSafePCM {}
unsafe impl Sync for ThreadSafePCM {}

struct AlsaInputDevice {
    pcm: ThreadSafePCM,
}

impl InputDevice for AlsaInputDevice {
    fn read(&mut self, buffer: &mut [i16]) -> Result<usize, BoxError> {
        // Direct access to pcm, no locking needed
        // Implement reading from ALSA device using self.pcm
        Err(format!("Not implemented, but here's the {:?}", buffer).into())
    }

    fn start(&mut self) -> Result<(), BoxError> {
        self.pcm.0.start()?;
        Ok(())
    }

    fn recover(&mut self, errno: i32) -> Result<(), BoxError> {
        self.pcm.0.recover(errno, true)?;
        Ok(())
    }
}

struct AlsaOutputDevice {
    pcm: ThreadSafePCM,
}

impl OutputDevice for AlsaOutputDevice {
    fn write(&mut self, buffer: &[i16]) -> Result<usize, BoxError> {
        // Implement writing to ALSA device
        // ...
        Err("Not implemented".into())
    }

    fn avail_update(&self) -> Result<i64, BoxError> {
        Ok(self.pcm.0.avail_update().map_err(|e| Box::new(e))? as i64)
    }

    fn recover(&mut self, errno: i32) -> Result<(), BoxError> {
        self.pcm.0.recover(errno, true)?;
        Ok(())
    }
}

// Define a stub JamEngine for testing purposes
// This is temp until a proper trait is created in JamEngine
struct JamEngine {
    running: bool,
}

impl JamEngine {
    pub fn new() -> Self {
        JamEngine { running: true }
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}

/// Represents a thread that handles ALSA audio processing.
/// 
/// This struct encapsulates the necessary components for managing ALSA audio input and output operations.
/// It includes a JamEngine for audio processing, input and output devices for ALSA operations, a handle to the thread,
/// a flag indicating if the thread is running, and statistics for stream timing.
/// 
/// To use this struct, create an instance of AlsaThread by calling `AlsaThread::new` or `AlsaThread::new_with_devices`.
/// The `new` method is a convenient way to create an AlsaThread instance with default ALSA devices specified by their names.
/// The `new_with_devices` method allows for more control by directly injecting the input and output devices.
/// 
/// Once an AlsaThread instance is created, call the `run` method to start the audio processing thread.
/// This method will spawn a new thread that will handle the audio processing tasks.
/// 
/// Note: The `run` method should be called only once for each instance of AlsaThread.
struct AlsaThread {
    indev_name: String,
    outdev_name: String,
    thread_handle: Option<std::thread::JoinHandle<()>>,
    stats: StreamTimeStat,
}

impl AlsaThread {
    pub fn new(in_device: String, out_device: String, engine: JamEngine) -> Result<Self, BoxError> {
        debug!("::new - contructing AlsaThread in: {}, out: {}", in_device, out_device);
        let input = Self::create_input_device(&in_device)?;
        let output = Self::create_output_device(&out_device)?;
        Ok(Self::new_with_devices(in_device, out_device, input, output, engine)?)
    }

    pub fn new_with_devices(in_device: String, out_device: String, input: Box<dyn InputDevice>, output: Box<dyn OutputDevice>, engine: JamEngine) -> Result<Self, BoxError> {
        debug!("::new - ALSA devices acquired, starting thread ...");
        let thread_handle = Self::run(engine, input, output)?;
        debug!("::new - thread creation success, returning to caller");

        Ok(AlsaThread {
            indev_name: in_device,
            outdev_name: out_device,
            thread_handle,
            stats: StreamTimeStat::new(100),
        })
    }

    fn create_input_device(device_name: &str) -> Result<Box<dyn InputDevice>, BoxError> {
        let pcm = PCM::new(device_name, Direction::Capture, false)?;
        {
            let hwp = HwParams::any(&pcm)?;
            hwp.set_channels(CHANNELS)?;
            hwp.set_rate(SAMPLE_RATE, ValueOr::Nearest)?;
            hwp.set_format(SMP_FORMAT)?;
            hwp.set_access(Access::RWInterleaved)?;
            hwp.set_buffer_size(2 * FRAME_SIZE as i64)?;
            hwp.set_period_size(FRAME_SIZE as i64, alsa::ValueOr::Nearest)?;
            pcm.hw_params(&hwp)?;
        }  // hwp is dropped here, releasing the bottow on &pcm, else it cannot be part of the return value
        pcm.start();
    
        debug!("::create_input_device - Opened audio input {} with parameters: {:?}, {:?}", 
               device_name, pcm.hw_params_current(), pcm.sw_params_current());
        Ok(Box::new(AlsaInputDevice { pcm: ThreadSafePCM(pcm) }))
    }

    fn create_output_device(device_name: &str) -> Result<Box<dyn OutputDevice>, BoxError> {
        let req_bufsize: i64 = (FRAME_SIZE * 4) as i64;  // A few ms latency by default, that should be nice
    
        // Open the device
        let pcm = alsa::PCM::new(device_name, alsa::Direction::Playback, false)?;
    
        {
            let hwp = HwParams::any(&pcm)?;
            hwp.set_channels(CHANNELS)?;
            hwp.set_rate(SAMPLE_RATE, alsa::ValueOr::Nearest)?;
            hwp.set_format(SMP_FORMAT)?;
            hwp.set_access(Access::MMapInterleaved)?;
            hwp.set_buffer_size(req_bufsize)?;
            hwp.set_period_size(req_bufsize / 4, alsa::ValueOr::Nearest)?;
            pcm.hw_params(&hwp)?;
            
            // Set software parameters
            //let hwp = pcm.hw_params_current()?;
            let swp = pcm.sw_params_current()?;
            let (bufsize, periodsize) = (hwp.get_buffer_size()?, hwp.get_period_size()?);
            swp.set_start_threshold(bufsize - periodsize)?;
            swp.set_avail_min(periodsize)?;
            pcm.sw_params(&swp)?;
            // let _rate = hwp.get_rate()?; // not sure why this would be needed
        }  // hwp is dropped here, releasing the bottow on &pcm, else it cannot be part of the return value
        
        debug!("::create_output_device - Opened audio output {:} with parameters: {:?}, {:?}",
            device_name, pcm.hw_params_current(), pcm.sw_params_current());   
        Ok(Box::new(AlsaOutputDevice { pcm: ThreadSafePCM(pcm) }))    
    }

    fn run(engine: JamEngine, input: Box<dyn InputDevice>, output: Box<dyn OutputDevice>) -> Result<Option<JoinHandle<()>>, BoxError> {
        let handle = std::thread::spawn(move || {
            Self::process_loop(engine, input, output);
        });

        Ok(Some(handle))
    }

    fn process_loop(engine: JamEngine, mut input: Box<dyn InputDevice>, mut output: Box<dyn OutputDevice>) {
        while engine.is_running() {
            let mut buffer = vec![0i16; FRAME_SIZE];
            if let Ok(bytes_read) = input.read(&mut buffer) {
                let _ = output.write(&buffer[..bytes_read]);
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.thread_handle.take() {
            handle.join().unwrap();
        }
    }

    pub fn is_running(&self) -> bool {
        self.thread_handle.is_some()
    }

    pub fn get_stats(&self) -> &StreamTimeStat {
        &self.stats
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    //use mockall::predicate::*;

    struct MockInputDevice {
        // Mocked input device fields
    }

    impl InputDevice for MockInputDevice {
        fn read(&mut self, buffer: &mut [i16]) -> Result<usize, BoxError> {
            // Mock implementation
            buffer.fill(0); // Fill buffer with zeros for testing
            Ok(buffer.len()) // Return the number of samples read
        }

        fn start(&mut self) -> Result<(), BoxError> {
            Ok(())
        }

        fn recover(&mut self, _errno: i32) -> Result<(), BoxError> {
            Ok(())
        }
    }

    struct MockOutputDevice {
        // Mocked output device fields
    }

    impl OutputDevice for MockOutputDevice {
        fn write(&mut self, buffer: &[i16]) -> Result<usize, BoxError> {
            // Mock implementation
            Ok(buffer.len()) // Return the number of samples written
        }

        fn avail_update(&self) -> Result<i64, BoxError> {
            Ok(0) // Mock available update
        }

        fn recover(&mut self, _errno: i32) -> Result<(), BoxError> {
            Ok(())
        }
    }

    #[test]
    fn test_alsa_thread_creation() {
        let engine = JamEngine::new(); // Assuming JamEngine has a new() method
        let input_device = Box::new(MockInputDevice {});
        let output_device = Box::new(MockOutputDevice {});

        let alsa_thread = AlsaThread::new_with_devices(
            "mock_input".to_string(),
            "mock_output".to_string(),
            input_device,
            output_device,
            engine,
        );

        assert!(alsa_thread.is_ok());
    }
}

