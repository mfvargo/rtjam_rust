//! map to dynamically assign people in the jam room to mixer slots
//!
//! as people come and go to the room, this map will assign them particular
//! mixer channels.  Think of this as the guy who runs the board assigning players
//! to slots on the board as they come and go.  "Oh Joe, here you are.  I'm gonna put
//! your vocals on channel 6 and your guitar on channel 7"
//!
//! ### TODO
//! this code has built in assumption that each "Player" has exactly two channels.  it
//! fills slots based on first available and the two channels are always adjacent.
use super::mixer::MIXER_CHANNELS;
use crate::common::player::{Player, EMPTY_SLOT};
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

// This is how long a player lasts until we boot them (if they go silent)
const NUM_PLAYERS_IN_ROOM: usize = MIXER_CHANNELS / 2 - 1; // take away one cause the local guy is always in the room

/// Map of the clients.
///
/// There is no specific "add" function.  when you search for an ID and it's
/// not in the map, it will be added for you as long as there is room
///
/// Note that the local user is always assigned slots 0 and 1.  Your channels
/// are always the first two on the mixer
pub struct ChannelMap {
    players: Vec<Player>,
}

impl ChannelMap {
    /// Build a map
    pub fn new() -> ChannelMap {
        let mut map = ChannelMap { players: vec![] };
        for _ in 0..NUM_PLAYERS_IN_ROOM {
            map.players.push(Player::new(
                0,
                EMPTY_SLOT,
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9999),
            ));
        }
        map
    }
    /// Mark all the slots as empty
    pub fn clear(&mut self) -> () {
        for p in &mut self.players {
            p.clear();
        }
    }
    /// see if any slots can be freed up (the guy left)
    pub fn prune(&mut self, now: u128) -> () {
        // search for aged clients
        for c in &mut self.players {
            if !c.is_empty() && c.is_old(now) {
                c.clear();
            }
        }
    }
    /// Get a list of the clients.  Used by the engine to dump out metadata to the u/x
    pub fn get_clients(&self) -> &[Player] {
        &self.players
    }
    /// retrieve the first channel on the mixer where this client is assigned (remember they come in pairs)
    pub fn get_loc_channel(&mut self, id: u32, now: u128, seq: u32) -> Option<usize> {
        // search for this id
        match self.players.iter().position(|c| c.client_id == id) {
            Some(idx) => {
                // Update the keepalive
                self.players[idx].update(now, id, 0, seq);
                Some((idx + 1) * 2)
            }
            None => {
                // Nobody found with that ID.  Get first available slot
                match self.players.iter().position(|p| p.is_empty()) {
                    Some(idx) => {
                        self.players[idx].update(now, id, 0, seq);
                        Some((idx + 1) * 2)
                    }
                    None => None,
                }
            }
        }
    }
}

impl fmt::Display for ChannelMap {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[\n")?;
        for c in &self.players {
            if !c.is_empty() {
                write!(f, "{}\n,", serde_json::to_string(&c).unwrap())?;
            }
        }
        write!(f, "]")
    }
}

#[cfg(test)]

mod test_channel_map {
    use crate::common::{get_micro_time, player::EXPIRATION_IN_MICROSECONDS};

    use super::*;

    #[test]
    fn find_a_slot() {
        let mut map = ChannelMap::new();
        let now = get_micro_time();
        let val = map.get_loc_channel(1234, now, 1).unwrap();
        assert_eq!(val, 2);
        let val_2 = map.get_loc_channel(4444, now, 1).unwrap();
        assert_eq!(val_2, 4);
        map.prune(now + EXPIRATION_IN_MICROSECONDS + 1);
    }
}
