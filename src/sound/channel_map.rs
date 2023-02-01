use super::mixer::MIXER_CHANNELS;
use std::fmt;

// This is how long a player lasts until we boot them (if they go silent)
const CLIENT_EXPIRATION_IN_MICROSECONDS: u128 = 1_000_000;
const NUM_PLAYERS_IN_ROOM: usize = MIXER_CHANNELS / 2 - 1; // take away one cause the local guy is always in the room

pub struct Client {
    pub client_id: u32,
    pub keep_alive: u128,
}

impl Client {
    pub fn new() -> Client {
        Client {
            client_id: 0,
            keep_alive: 0,
        }
    }
}

pub struct ChannelMap {
    clients: Vec<Client>,
}

impl ChannelMap {
    pub fn new() -> ChannelMap {
        let mut map = ChannelMap { clients: vec![] };
        for _ in 0..NUM_PLAYERS_IN_ROOM {
            map.clients.push(Client::new());
        }
        map
    }
    pub fn prune(&mut self, now: u128) -> () {
        // search for aged clients
        for c in &mut self.clients {
            if c.client_id > 0 && c.keep_alive + CLIENT_EXPIRATION_IN_MICROSECONDS < now {
                c.client_id = 0;
                c.keep_alive = 0;
            }
        }
    }
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
                match self.clients.iter().position(|c| c.client_id == 0) {
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
