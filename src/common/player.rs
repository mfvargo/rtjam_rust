use crate::dsp::smoothing_filter::SmoothingFilter;
use serde::Serialize;
use std::net::SocketAddr;

// This is how long a player lasts until we boot them (if they go silent)
const EXPIRATION_IN_MICROSECONDS: u128 = 1_000_000;
// represents an empty slot (legacy vst hangover)
const EMPTY_SLOT: u32 = 40000;
// represents maximum loop time that will be counted
pub const MAX_LOOP_TIME: u128 = 100_000;

/// Structure that represents a person in a room.  Used by both sound and broadcast components
///
/// It has an ID, (assigned by rtjam-nation when they join a room)
/// The keep_alive is used to time them out if we have not heard from them for over a second (no packets)
#[derive(Serialize)]
pub struct Player {
    address: SocketAddr,            // key used by broadcast
    client_id: u32,                 // key used by sound
    keep_alive: u128,               // last time we saw this player
    seq: u32,                       // packet sequence number
    drops: usize,                   // dropped packet counter
    hist: [usize; 10],              // histogram of packet arrivals
    pub loop_stat: SmoothingFilter, // statistics about packet frequency
}

impl Player {
    pub fn new(now_time: u128, id: u32, addr: SocketAddr) -> Player {
        Player {
            client_id: id,
            keep_alive: now_time,
            seq: 0,
            drops: 0,
            hist: [0; 10],
            address: addr,
            loop_stat: SmoothingFilter::build(0.5, 2666.6),
        }
    }
    pub fn get_drops(&self) -> usize {
        self.drops
    }
    pub fn clear(&mut self) -> () {
        self.hist = [0; 10];
        self.client_id = EMPTY_SLOT;
        self.keep_alive = 0;
        self.seq = 0;
        self.drops = 0;
    }
    pub fn update(&mut self, now: u128, id: u32, loop_time: u128, seq: u32) -> () {
        if self.keep_alive <= now {
            let idx: usize = ((now - self.keep_alive) / 2667) as usize; // 2910 microsec per 128 sample frame
            self.hist[idx.clamp(0, 9)] += 1;
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
