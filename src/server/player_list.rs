use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::stream_time_stat::StreamTimeStat;
use serde::{Deserialize, Serialize};

// Get the time in microseconds
pub fn get_micro_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros()
}

// This is how long a player lasts until we boot them (if they go silent)
const SERVER_EXPIRATION_IN_MICROSECONDS: u128 = 500000;

// make the player serialize to json
#[derive(Debug, Deserialize, Serialize)]
pub struct Player {
    pub client_id: u32,
    pub keep_alive: u128,
    pub last_loop: u128,
    pub address: SocketAddr,
    pub loop_stat: StreamTimeStat,
}

impl Player {
    pub fn build(now_time: u128, id: u32, addr: SocketAddr) -> Player {
        Player {
            client_id: id,
            keep_alive: now_time,
            last_loop: now_time,
            address: addr.clone(),
            loop_stat: StreamTimeStat::build(50),
        }
    }
    pub fn age(&self, now_time: u128) -> u128 {
        now_time - self.keep_alive
    }
}

impl fmt::Display for Player {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ id: {}, address: {}, age: {} loop_time: {} }}",
            self.client_id,
            self.address,
            self.age(get_micro_time()),
            self.last_loop
        )
    }
}

#[cfg(test)]
mod test_player {
    use std::net::IpAddr;
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn build_player() {
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let player = Player::build(get_micro_time(), 44, socket);
        println!("player: {}", player);
        assert_eq!(player.address, socket);
        println!("json player: {}", serde_json::to_string(&player).unwrap());
    }
}
pub struct PlayerList {
    pub players: Vec<Player>,
}

impl PlayerList {
    pub fn build() -> PlayerList {
        PlayerList { players: vec![] }
    }
    pub fn is_allowed(&self, _id: u32) -> bool {
        true
    }
    pub fn update_player(
        &mut self,
        now_time: u128,
        loop_time: u128,
        id: u32,
        addr: SocketAddr,
    ) -> () {
        // look for this player and update their timestamp if found
        for player in &mut self.players {
            if player.address == addr {
                player.keep_alive = now_time;
                player.address = addr;
                player.client_id = id;
                if loop_time < 5000 {
                    player.last_loop = loop_time;
                    player.loop_stat.add_sample(loop_time as f64);
                }
                return ();
            }
        }
        // If we got here, we don't know this guy.  add him
        self.players.push(Player::build(now_time, id, addr));
    }
    pub fn prune(&mut self, now_time: u128) -> () {
        // this function will age out any old Players
        self.players
            .retain(|p| p.keep_alive + SERVER_EXPIRATION_IN_MICROSECONDS > now_time);
    }
    pub fn get_players(&self) -> &Vec<Player> {
        &self.players
    }

    pub fn get_latency(&mut self) -> serde_json::Value {
        let mut lmap: HashMap<u32, f64> = HashMap::new();
        for p in &self.players {
            lmap.insert(p.client_id, p.loop_stat.get_mean());
        }
        serde_json::json!({
            "latency": lmap,
        })
    }
}

impl fmt::Display for PlayerList {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ ")?;
        for player in &self.players {
            write!(f, " {},", player)?;
        }
        write!(f, " ]")
    }
}
#[cfg(test)]
mod test_playerlist {
    use super::*;

    #[test]
    fn build() {
        // you should be able to build a PlayerList
        let plist = PlayerList::build();
        println!("plist: {}", plist.players.len());
        assert!(true);
    }
    #[test]
    fn is_allowed() {
        // TODO:  no id verification yet.  Must implement ability to filter data to just those
        // clients who have joined the room on the server
        let plist = PlayerList::build();
        assert_eq!(plist.is_allowed(44455), true);
    }
    #[test]
    fn update_player() {
        // functions to add/update players to the list
        let mut plist = PlayerList::build();
        let now_time = get_micro_time();
        let loop_time = now_time - 2400;
        let id = 55533;
        let addr: SocketAddr = "182.1.1.1:33345"
            .parse()
            .expect("Unable to parse socket address");
        // Add a new player to an empty list
        plist.update_player(now_time, loop_time, id, addr);
        assert_eq!(plist.get_players().len(), 1);
        // this will update a player if we have seen them before
        plist.update_player(now_time + 100, loop_time, id, addr);
        assert_eq!(plist.get_players().len(), 1);
        // This will add another player
        let addr: SocketAddr = "192.1.1.1:33345"
            .parse()
            .expect("Unable to parse socket address");
        plist.update_player(now_time, loop_time, id, addr);
        assert_eq!(plist.get_players().len(), 2);
    }
    #[test]
    fn prune() {
        // This function will age out a player when they get too old
        let mut plist = PlayerList::build();
        let now_time = get_micro_time();
        let id = 55533;
        let addr: SocketAddr = "182.1.1.1:33345"
            .parse()
            .expect("Unable to parse socket address");
        // Add a new player to an empty list
        plist.update_player(now_time, now_time, id, addr);
        assert_eq!(plist.get_players().len(), 1);
        // Call prune with a now_time that is past
        plist.prune(now_time + SERVER_EXPIRATION_IN_MICROSECONDS + 1);
        assert_eq!(plist.get_players().len(), 0);
    }
    #[test]
    fn get_latency() {
        let mut plist = PlayerList::build();
        let now_time = get_micro_time();
        let id = 55533;
        let addr: SocketAddr = "182.1.1.1:33345"
            .parse()
            .expect("Unable to parse socket address");
        // Add a new player to an empty list
        plist.update_player(now_time, now_time, id, addr);
        println!("latency: {}", plist.get_latency());
    }
}