//! List of current players in the room.  Used to multicast to them
//!
//! The broadcast component will add/remove sound components to the room using
//! this list.
use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;

use crate::common::player::Player;

/// Structure to hold the list of players
pub struct PlayerList {
    pub players: Vec<Player>,
    pub stat_queue: Vec<serde_json::Value>,
}

impl PlayerList {
    pub fn new() -> PlayerList {
        PlayerList {
            players: vec![],
            stat_queue: vec![],
        }
    }
    /// tell if the player is allowed in the room
    ///
    /// TODO: Needs to be implemented.  Right now everybody is allowed.  Need to have
    /// a list of allowed players that will be retrieved from rtjam-nation
    pub fn is_allowed(&self, _id: u32) -> bool {
        true
    }
    /// update the keepalive for this player (found by ip address)
    ///
    /// called when we receive a packet from a player
    pub fn update_player(
        &mut self,
        now_time: u128,
        loop_time: u128,
        id: u32,
        addr: SocketAddr,
        seq: u32,
    ) -> () {
        // look for this player and update their timestamp if found
        for player in &mut self.players {
            if player.address == addr {
                player.update(now_time, id, loop_time, seq);
                return ();
            }
        }
        // If we got here, we don't know this guy.  add him
        self.players.push(Player::new(now_time, id, addr));
    }
    /// look for any player entries that have timed out
    pub fn prune(&mut self, now_time: u128) -> () {
        for p in &self.players {
            // save stats for values about to be cleared
            if p.is_old(now_time) {
                match serde_json::to_value(p) {
                    Ok(v) => self.stat_queue.push(v),
                    Err(_e) => (),
                }
            }
        }
        // this function will age out any old Players
        self.players.retain(|p| !p.is_old(now_time));
    }
    /// Get a list of players to iterate though
    pub fn get_players(&self) -> &Vec<Player> {
        &self.players
    }

    /// Get a json representation of the players in the room and their current latency
    ///
    /// used to update other players in the room about the status of everybody's connection
    pub fn get_latency(&mut self) -> serde_json::Value {
        let mut lmap: HashMap<u32, f64> = HashMap::new();
        for p in &self.players {
            lmap.insert(p.client_id, p.get_last_loop().round() / 1000.0);
            // Convert to msec
        }
        serde_json::json!({
            "latency": lmap,
        })
    }
}

impl fmt::Display for PlayerList {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "active: [\n")?;
        for player in &self.players {
            write!(f, " {},\n", player)?;
        }
        write!(f, " ]\nold: [\n")?;
        for stat in &self.stat_queue {
            write!(f, " {},\n", stat.to_string())?;
        }
        write!(f, " ]")
    }
}
#[cfg(test)]
mod test_playerlist {
    use super::*;
    use crate::common::{get_micro_time, player::EXPIRATION_IN_MICROSECONDS};

    #[test]
    fn build() {
        // you should be able to build a PlayerList
        let plist = PlayerList::new();
        println!("plist: {}", plist.players.len());
        assert!(true);
    }
    #[test]
    fn is_allowed() {
        // TODO:  no id verification yet.  Must implement ability to filter data to just those
        // clients who have joined the room on the server
        let plist = PlayerList::new();
        assert_eq!(plist.is_allowed(44455), true);
    }
    #[test]
    fn update_player() {
        // functions to add/update players to the list
        let mut plist = PlayerList::new();
        let now_time = get_micro_time();
        let loop_time = now_time - 2400;
        let id = 55533;
        let addr: SocketAddr = "182.1.1.1:33345"
            .parse()
            .expect("Unable to parse socket address");
        // Add a new player to an empty list
        plist.update_player(now_time, loop_time, id, addr, 0);
        assert_eq!(plist.get_players().len(), 1);
        // this will update a player if we have seen them before
        plist.update_player(now_time + 100, loop_time, id, addr, 0);
        assert_eq!(plist.get_players().len(), 1);
        // This will add another player
        let addr: SocketAddr = "192.1.1.1:33345"
            .parse()
            .expect("Unable to parse socket address");
        plist.update_player(now_time, loop_time, id, addr, 0);
        assert_eq!(plist.get_players().len(), 2);
    }
    #[test]
    fn prune() {
        // This function will age out a player when they get too old
        let mut plist = PlayerList::new();
        let now_time = get_micro_time();
        let id = 55533;
        let addr: SocketAddr = "182.1.1.1:33345"
            .parse()
            .expect("Unable to parse socket address");
        // Add a new player to an empty list
        plist.update_player(now_time, now_time, id, addr, 0);
        assert_eq!(plist.get_players().len(), 1);
        // Call prune with a now_time that is past
        plist.prune(now_time + EXPIRATION_IN_MICROSECONDS + 1);
        assert_eq!(plist.get_players().len(), 0);
    }
    #[test]
    fn get_latency() {
        let mut plist = PlayerList::new();
        let now_time = get_micro_time();
        let id = 55533;
        let addr: SocketAddr = "182.1.1.1:33345"
            .parse()
            .expect("Unable to parse socket address");
        // Add a new player to an empty list
        plist.update_player(now_time, now_time, id, addr, 0);
        println!("latency: {}", plist.get_latency());
    }
}
