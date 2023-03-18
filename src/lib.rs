//! rtjam - Real Time Jam library
//!
//! provides library elements to create code for a jamUnit (device to connect with)
//! and a broadcast_server which will host rooms for real time audio conferencing
//! # RTJam
//!
//! Real Time Music Jam
//!
//! The Real Time Music Jam software is intended to provide low latency audio over the internet that will enable performers to make music as if they were in the same place. The system is comprised of a broadcast server that listens on a UDP port for jam packets and some number of clients that create/consume audio data.
//!
//! The server keeps a table of jammers in the "room" and will forward each packet it receives to the other jammers in the room. It does not transmit the audio data back to the orginator (client provides local monitoring).
//!
//! Each jam packet has two separate channels of audio so the jammers can have a channel for voice and guitar, stereo keyboards, or whatever they choose. The two channels are isochronous for that person.
//!
//! So in this way a room consists of jammers each with two channels.
//!
//! ## Get The code
//!
//! ```text
//! git clone https://github.com/mfvargo/rtjam_rust.git
//! cd rtjam_rust
//! cargo build --package rtjam_rust --example rtjam_broadcast --example rtjam_sound --release
//! ```
//!
//! ## Components
//!
//! ### rtjam_rust library
//!
//! This is the library of all the components used to make the rtjam_sound and rtjam_broadcast executables.
//!
//! ### Jack Standalone (examples/rtjam_sound.rs)
//!
//! The RTJam software also builds on the Raspberry Pi 4 and can be run as a "standalone JACK" application. This has been the most successful implementation for playing music realtime on the internet. The Pi has a very stable multimedia jack port that can run 64 sample frames with only 2 period of buffer without the dreaded XRUN issues you see on most other platforms.
//!
//! Although the pi is a good platform, the software will also run on Mac OSX and Windows as well as linux.
//! The requirements for this to run are that jackd2 is installed. The underlying hardware needs to support 128 sample/frame at 48000 Hz sample rate.
//!
//! ### Broadcast Server (examples/rtjam_broadcast.rs)
//!
//! The server just listens for packets from rtjam clients. The server dynamically creates channels based on the source address of the client packets and forwards packets to all active listeners. There is currently no session control. When you start transmitting the server will allocate a channel to you if one is open. If you don't send any packets for 1 second, your channel is made available.
//!
//! The server should run on any OS that has a std rust compiler.
//!
extern crate json;
#[macro_use]
extern crate num_derive;

pub use self::pedals::pedal::Pedal;
pub use self::sound::jam_engine::JamEngine;
pub use self::sound::param_message::ParamMessage;

pub mod common;
pub mod dsp;
pub mod pedals;
pub mod server;
pub mod sound;
pub mod utils;
