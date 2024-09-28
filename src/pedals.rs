//! effect pedals that can be chained into a pedal board
//!
//! Pedals can be added to the processing stream of the client software.  This is
//! done by having all pedals implement the [`Pedal`](crate::pedals::pedal::Pedal) trait.
//! the trait provides the top down look to the different pedal implementation.  This way the
//! [`PedalBoard`](crate::pedals::pedal_board::PedalBoard) can have an arbitrary set of
//! pedals connected in a chain.
pub mod bass_di;
pub mod bass_envelope;
pub mod chorus;
pub mod compressor;
pub mod controls;
pub mod delay;
pub mod delay_base;
pub mod distortion_base;
pub mod envelope_base;
pub mod guitar_envelope;
pub mod noise_gate;
pub mod pedal;
pub mod pedal_board;
pub mod sigma_reverb;
pub mod soul_drive;
pub mod speaker_sim_iir;
pub mod tone_stack;
pub mod tremelo;
pub mod tube_drive;
pub mod champ;
pub mod princeton;
pub mod tube_screamer;
pub mod template_pedal;
