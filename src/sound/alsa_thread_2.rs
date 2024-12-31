//use std::f64::consts::E;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use alsa::pcm::*;
use alsa::{Direction, ValueOr};

#[cfg(test)]
use mockall::automock;

// use crate::JamEngine;
use crate::common::box_error::BoxError;
use crate::common::stream_time_stat::{/*MicroTimer,*/ StreamTimeStat};
//use crate::JamEngine;

use log::{ debug, trace, info, warn, /* error */};

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
struct ThreadSafePCM(Arc<Mutex<PCM>>);
unsafe impl Send for ThreadSafePCM {}
unsafe impl Sync for ThreadSafePCM {}

struct AlsaInputDevice {
    pcm: ThreadSafePCM,
}

impl InputDevice for AlsaInputDevice {
    fn read(&mut self, buffer: &mut [i16]) -> Result<usize, BoxError> {
        let pcm_guard = self.pcm.0.lock().unwrap();
        let status = pcm_guard.status();
        Err(format!("Not implemented, but here's the status: {} and the buffer {:?}", status.is_ok(), buffer).into())
    }

    fn start(&mut self) -> Result<(), BoxError> {
        let pcm_guard = self.pcm.0.lock().unwrap();
        let _status = pcm_guard.start()?;
        Ok(())
    }

    fn recover(&mut self, errno: i32) -> Result<(), BoxError> {
        let pcm_guard = self.pcm.0.lock().unwrap();
        let _status = pcm_guard.recover(errno, true)?;
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
        let pcm_guard = self.pcm.0.lock().unwrap();
        let status = pcm_guard.status();
        Err(format!("Not implemented, but here's the status {} and the buffer {:?}", status.is_ok(), buffer).into())
    }

    fn avail_update(&self) -> Result<i64, BoxError> {
        let pcm_guard = self.pcm.0.lock().unwrap();
        Ok(pcm_guard.avail_update().map_err(|e| Box::new(e))? as i64)
    }

    fn recover(&mut self, errno: i32) -> Result<(), BoxError> {
        let pcm_guard = self.pcm.0.lock().unwrap();
        let _status = pcm_guard.recover(errno, true)?;
        Ok(())
    }
}

// Define a stub JamEngine for testing purposes
// This is temp until a proper trait is created in JamEngine
trait AlsaEngineTrait: Send + Sync {
    fn is_running(&self) -> bool;
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
struct AlsaThread<I: InputDevice, O: OutputDevice, E: AlsaEngineTrait> {
    indev_name: String,
    outdev_name: String,
    indev: Arc<Mutex<Box<I>>>,
    outdev: Arc<Mutex<Box<O>>>,
    engine: Arc<E>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
    stats: StreamTimeStat,
}

impl <I: InputDevice + 'static, O: OutputDevice + 'static, E: AlsaEngineTrait + 'static> AlsaThread<I, O, E> {

    pub fn build(indev_name: String, outdev_name: String, engine: E) -> Result<AlsaThread<I, O, E>, BoxError> {
        debug!("::build - contructing AlsaThread in: {}, out: {}", indev_name, outdev_name);
        info!("Starting ALSA thread");
        let input: AlsaInputDevice = Self::create_input_device(&indev_name)?;
        let output: AlsaOutputDevice = Self::create_output_device(&outdev_name)?;
        let mut alsa: AlsaThread<I, O, E> = Self::new(indev_name, outdev_name, input, output, engine)?;
        let _run_result = alsa.run()?;
        Ok(alsa)
    }

    pub fn new(
        indev_name: String, outdev_name: String, 
        input: I, output: O, 
        engine: E,
    ) -> Result<Self, BoxError> {
        debug!("::new - ALSA devices acquired, testing comms ...");
        // TODO: Find a better way to validate the PCMs are alive than a blank read and write
        //Self::read_in_write_out(1, input.clone(), output.clone())?;

        debug!("::new - ALSA devices acquired, starting thread ...");
        let indev = Arc::new(Mutex::new(Box::new(input)));
        let outdev = Arc::new(Mutex::new(Box::new(output)));
        let engine_arc = Arc::new(engine);

        let alsa_thread = AlsaThread {
            indev_name,
            outdev_name,
            indev,
            outdev,
            engine: engine_arc,
            thread_handle: None,
            stats: StreamTimeStat::new(100),       
        };
        Ok(alsa_thread)
    }

    fn create_input_device(device_name: &str) -> Result<AlsaInputDevice, BoxError> {
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
        pcm.start()?;
    
        debug!("::create_input_device - Opened audio input {} with parameters: {:?}, {:?}", 
               device_name, pcm.hw_params_current(), pcm.sw_params_current());
        Ok(AlsaInputDevice { pcm: ThreadSafePCM(Arc::new(Mutex::new(pcm))) })
    }

    fn create_output_device(device_name: &str) -> Result<AlsaOutputDevice, BoxError> {
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
        Ok(AlsaOutputDevice { pcm: ThreadSafePCM(Arc::new(Mutex::new(pcm))) })    
    }

    fn run(&mut self) -> Result<(), BoxError> {
        debug!("::run - starting. Spawning thread now ...");
        // Clone arcs once here and not repeatedly in the loop
        let in_clone = self.indev.clone();
        let out_clone = self.outdev.clone();
        let engine_arc = self.engine.clone();

        let handle = std::thread::spawn( move || {
            debug!("::process_loop - starting. Engine is running: {}", engine_arc.is_running());
            while engine_arc.is_running() {
                trace!("::process_loop - calling read_in_write_out");
                let _result = Self::read_in_write_out(&in_clone, &out_clone, FRAME_SIZE);
            };
        });

        debug!("::run - thread spawned successfully. Handle: {:?}", handle);
        self.thread_handle = Some(handle);
        Ok(())
    }
    
    fn read_in_write_out(indev: &Arc<Mutex<Box<I>>>, outdev: &Arc<Mutex<Box<O>>>, size: usize) -> Result<usize, BoxError> {
        debug!("::read_in_write_out - frame size: {}", size);
        let mut buffer = vec![0i16; size];

        let mut in_guard = indev.lock().unwrap();
        let bytes_read = in_guard.read(&mut buffer)?;
        trace!("::read_in_write_out - read {} bytes. Writing to output ...", bytes_read);

        let mut out_guard = outdev.lock().unwrap();
        let bytes_written = out_guard.write(&buffer[..bytes_read])?;

        Ok(bytes_written)
    }

    fn self_read_in_write_out(&mut self, size: usize) -> Result<usize, BoxError> {
        debug!("::read_in_write_out - frame size: {}", size);
        let mut buffer = vec![0i16; size];
        //let mut bytes_read = 0;
        //let mut bytes_written = 0;

        let mut in_guard = self.indev.lock().unwrap();
        let bytes_read = in_guard.read(&mut buffer)?;
        trace!("::read_in_write_out - read {} bytes. Writing to output ...", bytes_read);
        //     Ok(b) => {
        //         //bytes_read = b;
        //         trace!("::read_in_write_out - read {} bytes. Writing to output ...", b);
        //         b
        //     },
        //     Err(_e) => {
        //         warn!("Error reading ALSA input");
        //         Err(BoxError::from("Error reading ALSA input"))
        //     }
        // }?;
    
        let mut out_guard = self.outdev.lock().unwrap();
        let bytes_written = out_guard.write(&buffer[..bytes_read])?;
        //     Ok(b) => {  
        //         bytes_written = b;          
        //         trace!("::read_in_write_out - wrote {} bytes", bytes_written);
        //     },
        //     Err(e) => {
        //         warn!("Error writing ALSA output");
        //         return Err(BoxError::from("Error writing ALSA output"));
        //     }
        // };
        Ok(bytes_written)
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
    //use alsa::Error;
    use log::{info, LevelFilter};
    use env_logger::Builder;
    // use std::sync::mpsc::channel;
    // use std::thread;

    use super::*;
    //use mockall::predicate::*;

    struct MockInputDevice {
        read_result: Result<usize, BoxError>,
        start_result: Result<(), BoxError>,
        recover_result: Result<(), BoxError>,
    }

    impl InputDevice for MockInputDevice {
        fn read(&mut self, buffer: &mut [i16]) -> Result<usize, BoxError> {
            match &self.read_result {
                Ok(sz) => Ok(*sz),
                Err(_) => Err(BoxError::from("Mock error")),
            }            // Mock implementation
            // buffer.fill(0); // Fill buffer with zeros for testing
            // Ok(buffer.len()) // Return the number of samples read
        }

        fn start(&mut self) -> Result<(), BoxError> {
            match &self.start_result {
                Ok(_) => Ok(()),
                Err(_) => Err(BoxError::from("Mock error")),
            }
        }

        fn recover(&mut self, _errno: i32) -> Result<(), BoxError> {
            match &self.recover_result {
                Ok(_) => Ok(()),
                Err(_) => Err(BoxError::from("Mock error")),
            }
        }
    }

    struct MockOutputDevice {
        write_result: Result<usize, BoxError>,
        avail_update_result: Result<i64, BoxError>,
        recover_result: Result<(), BoxError>,
    }

    impl OutputDevice for MockOutputDevice {
        fn write(&mut self, _buffer: &[i16]) -> Result<usize, BoxError> {
            match &self.write_result {
                Ok(b) => Ok(*b),
                Err(_) => Err(BoxError::from("Mock error")),
            }
        }

        fn avail_update(&self) -> Result<i64, BoxError> {
            match &self.avail_update_result {
                Ok(r) => Ok(*r),
                Err(_) => Err(BoxError::from("Mock error")),
            }        }

        fn recover(&mut self, _errno: i32) -> Result<(), BoxError> {
            match &self.recover_result {
                Ok(_) => Ok(()),
                Err(_) => Err(BoxError::from("Mock error")),
            }
        }
    }

    struct TestEngine {
        running: bool,
    }

    impl TestEngine {
        pub fn new() -> Self {
            TestEngine { running: true }
        }
    }

    impl AlsaEngineTrait for TestEngine {
        fn is_running(&self) -> bool {
            self.running
        }
    }

    fn log_init(level: LevelFilter) {
        let _ = Builder::new()
            .filter_level(level)
            .target(env_logger::Target::Stdout)
            .try_init();
        info!("Test logger initialized");
    }

    #[test]
    fn test_alsa_thread_creation() {
        log_init(LevelFilter::Debug);
        let engine = TestEngine::new();
        let input_device = Arc::new(MockInputDevice {
            read_result: Ok(127),
            start_result: Ok(()),
            recover_result: Ok(()),
        });
        let output_device = Arc::new(MockOutputDevice {
            write_result: Ok(0),
            avail_update_result: Ok(0),
            recover_result: Ok(()),
        });

        let result = AlsaThread::new_with_devices(
            "mock input".to_string(),
            "mock output".to_string(),
            input_device,
            output_device,
            engine,
        );

        assert!(result.is_ok());
        
        let alsa_thread = result.unwrap();
        assert_eq!(alsa_thread.indev_name, "mock input");
        assert_eq!(alsa_thread.outdev_name, "mock output");
        assert!(alsa_thread.thread_handle.is_some());
        assert_eq!(alsa_thread.stats.get_window(), 100);
    }

    #[test]
    fn test_alsa_thread_creation_start_fail() {
        log_init(LevelFilter::Trace);
        let engine = TestEngine::new();
        // If bypassing the device creation and passed in bad devices, the first likely error will be on an attempt to read indev
        let input_device = Arc::new(MockInputDevice {
            read_result: Err(Box::from("dummy value")),
            start_result: Ok(()),
            recover_result: Ok(()),
        });
        let output_device = Arc::new(MockOutputDevice {
            write_result: Ok(0),
            avail_update_result: Ok(0),
            recover_result: Ok(()),
        });

        let result = AlsaThread::new_with_devices(
            "mock input".to_string(),
            "mock output".to_string(),
            input_device,
            output_device,
            engine,
        );
        assert!(result.is_err());
    }
}

