use crate::{common::box_error::BoxError, JamEngine};

use super::alsa_device::AlsaDevice;



// Run the loop to read/write alsa
pub fn run(mut engine: JamEngine, in_device: &str, out_device: &str) -> Result<(), BoxError> {
    // Create alsa device
    let mut alsa_device = AlsaDevice::new(&in_device, &out_device)?;

    // Run the device
    while engine.is_running() {
        // process a frame of audio
        alsa_device.process_a_frame(&mut engine)?;
    }
    Ok(())
}
