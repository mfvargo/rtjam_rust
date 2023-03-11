//! map to dynamically assign people in the jam room to mixer slots
//!
//! as people come and go to the room, this map will assign them particular
//! mixer channels.  Think of this as the guy who runs the board assigning players
//! to slots on the board as they come and go.  "Oh Joe, here you are.  I'm gonna put
//! your vocals on channel 6 and your guitar on channel 7"
//!
//! ### TODO
//! this code has built in assumption that each "Client" has exactly two channels.  it
//! fills slots based on first available and the two channels are always adjacent.
use super::mixer::MIXER_CHANNELS;
use std::fmt;

// This is how long a player lasts until we boot them (if they go silent)
const CLIENT_EXPIRATION_IN_MICROSECONDS: u128 = 1_000_000;
const NUM_PLAYERS_IN_ROOM: usize = MIXER_CHANNELS / 2 - 1; // take away one cause the local guy is always in the room
const EMPTY_SLOT: u32 = 40000;

/// Structure that reprensents a Client
///
/// It has an ID, (assigned by rtjam-nation when they join a room)
/// The keep_alive is used to time them out if we have not heard from them for over a second (no packets)
pub struct Client {
    pub client_id: u32,
    pub keep_alive: u128,
}

impl Client {
    pub fn new() -> Client {
        Client {
            client_id: EMPTY_SLOT,
            keep_alive: 0,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.client_id == EMPTY_SLOT
    }
}

/// Map of the clients.
///
/// There is no specific "add" function.  when you search for an ID and it's
/// not in the map, it will be added for you as long as there is room
///
/// Note that the local user is always assigned slots 0 and 1.  Your channels
/// are always the first two on the mixer
pub struct ChannelMap {
    clients: Vec<Client>,
}

impl ChannelMap {
    /// Build a map
    pub fn new() -> ChannelMap {
        let mut map = ChannelMap { clients: vec![] };
        for _ in 0..NUM_PLAYERS_IN_ROOM {
            map.clients.push(Client::new());
        }
        map
    }
    /// Mark all the slots as empty
    pub fn clear(&mut self) -> () {
        for c in &mut self.clients {
            c.client_id = EMPTY_SLOT;
            c.keep_alive = 0;
        }
    }
    /// see if any slots can be freed up (the guy left)
    pub fn prune(&mut self, now: u128) -> () {
        // search for aged clients
        for c in &mut self.clients {
            if c.client_id > 0 && c.keep_alive + CLIENT_EXPIRATION_IN_MICROSECONDS < now {
                c.client_id = EMPTY_SLOT;
                c.keep_alive = 0;
            }
        }
    }
    /// Get a list of the clients.  Used by the engine to dump out metadata to the u/x
    pub fn get_clients(&self) -> &[Client] {
        &self.clients
    }
    /// retrieve the first channel on the mixer where this client is assigned (remember they come in pairs)
    pub fn get_loc_channel(&mut self, id: u32, now: u128) -> Option<usize> {
        // search for this id
        match self.clients.iter().position(|c| c.client_id == id) {
            Some(idx) => {
                // Update the keepalive
                self.clients[idx].keep_alive = now;
                Some((idx + 1) * 2)
            }
            None => {
                // Nobody found with that ID.  Get first available slot
                match self.clients.iter().position(|c| c.client_id == EMPTY_SLOT) {
                    Some(idx) => {
                        self.clients[idx].client_id = id;
                        self.clients[idx].keep_alive = now;
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
        write!(f, "[ ")?;
        for c in &self.clients {
            write!(f, "(id: {}, keep_alive: {}),", c.client_id, c.keep_alive)?;
        }
        write!(f, " ]")
    }
}

#[cfg(test)]

mod test_channel_map {
    use crate::server::player_list::get_micro_time;

    use super::*;

    #[test]
    fn find_a_slot() {
        let mut map = ChannelMap::new();
        let now = get_micro_time();
        let val = map.get_loc_channel(1234, now).unwrap();
        assert_eq!(val, 2);
        let val_2 = map.get_loc_channel(4444, now).unwrap();
        assert_eq!(val_2, 4);
        map.prune(now + CLIENT_EXPIRATION_IN_MICROSECONDS + 1);
        println!("after prune: {}", map);
    }
}
