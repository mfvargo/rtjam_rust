//! Struct use to track a players packet stream
//!
//! used by both sound and broadcast components.  
use crate::dsp::smoothing_filter::SmoothingFilter;
use serde::Serialize;
use std::fmt;
use std::net::SocketAddr;

use super::stream_time_stat::StreamTimeStat;

// This is how long a player lasts until we boot them (if they go silent)
pub const EXPIRATION_IN_MICROSECONDS: u128 = 1_000_000;
// represents an empty slot (legacy vst hangover)
pub const EMPTY_SLOT: u32 = 40000;
// represents maximum loop time that will be counted
pub const MAX_LOOP_TIME: u128 = 100_000;
// number of histogram buckets
const HISTOGRAM_BUCKETS: usize = 15;

/// Structure that represents a person in a room.  Used by both sound and broadcast components
///
/// It has an ID, (assigned by rtjam-nation when they join a room)
/// It also has a SocketAddr used by the broadcast server to do the multicast
/// The keep_alive is used to time them out if we have not heard from them for over a second (no packets)
/// It has the sequence number assigned by the packet originator and uses that to track dropped packets
/// lastly it has some stat objects to characterize the packet stream (histogram, loop stats and packet arrival stats)
#[derive(Serialize)]
pub struct Player {
    pub address: SocketAddr,          // key used by broadcast
    pub client_id: u32,               // key used by sound
    keep_alive: u128,                 // last time we saw this player
    seq: u32,                         // packet sequence number
    drops: usize,                     // dropped packet counter
    hist: [usize; HISTOGRAM_BUCKETS], // histogram of packet arrivals
    loop_stat: SmoothingFilter,       // statistics about packet loop time
    pack_stats: StreamTimeStat,       // interarrival stats
}

impl Player {
    pub fn new(now_time: u128, id: u32, addr: SocketAddr) -> Player {
        Player {
            client_id: id,
            keep_alive: now_time,
            seq: 0,
            drops: 0,
            hist: [0; HISTOGRAM_BUCKETS],
            address: addr,
            loop_stat: SmoothingFilter::build(0.5, 2666.6),
            pack_stats: StreamTimeStat::new(100),
        }
    }
    pub fn get_drops(&self) -> usize {
        self.drops
    }
    pub fn get_last_loop(&self) -> f64 {
        self.loop_stat.get_last_output()
    }
    pub fn clear(&mut self) -> () {
        self.hist = [0; HISTOGRAM_BUCKETS];
        self.client_id = EMPTY_SLOT;
        self.keep_alive = 0;
        self.seq = 0;
        self.drops = 0;
    }
    pub fn update(&mut self, now: u128, id: u32, loop_time: u128, seq: u32) -> () {
        if self.keep_alive <= now {
            self.pack_stats.add_sample((now - self.keep_alive) as f64);
            let idx: usize = ((now - self.keep_alive) / 2667) as usize; // 2667 microsec per 128 sample frame
            self.hist[idx.clamp(0, HISTOGRAM_BUCKETS - 1)] += 1;
        }
        if loop_time < MAX_LOOP_TIME {
            // Only count loop times less than 100msec
            self.loop_stat.get(loop_time as f64);
        }
        self.keep_alive = now;
        self.client_id = id;
        // Check sequence number
        if self.seq + 1 != seq {
            // We have a dropped packet
            self.drops += 1;
        }
        self.seq = seq;
    }
    pub fn is_old(&self, now: u128) -> bool {
        self.keep_alive + EXPIRATION_IN_MICROSECONDS < now
    }
    pub fn is_empty(&self) -> bool {
        self.client_id == EMPTY_SLOT
    }
}

impl fmt::Display for Player {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}
#[cfg(test)]
mod test_player {
    use crate::common::get_micro_time;
    use std::net::IpAddr;
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn build_player() {
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let player = Player::new(get_micro_time(), 44, socket);
        println!("player: {}", serde_json::to_string(&player).unwrap());
        assert_eq!(player.address, socket);
    }
}
