//! These modules are shared among both the client and server executables for rtjam.
//!

use std::time::{SystemTime, UNIX_EPOCH};
// Get the time in microseconds
pub fn get_micro_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros()
}

pub mod box_error;
pub mod config;
pub mod jam_nation_api;
pub mod jam_packet;
pub mod packet_stream;
pub mod player;
pub mod room;
pub mod sock_with_tos;
pub mod stream_time_stat;
pub mod websock_message;
pub mod websocket;
