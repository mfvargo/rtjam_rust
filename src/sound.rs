//! components used to make the rtjam_sound client

use crate::common::box_error::BoxError;

pub trait SoundCallback {
    fn process(&mut self, in_a: &[f32], in_b: &[f32], out_a: &mut [f32], out_b: &mut [f32]) -> Result<(), BoxError>;
}


pub mod channel_map;
pub mod channel_strip;
pub mod client;
pub mod fader;
pub mod alsa_thread;
pub mod alsa_device;
pub mod jack_thread;
pub mod jam_engine;
pub mod jam_socket;
pub mod jitter_buffer;
pub mod mixer;
pub mod param_message;
