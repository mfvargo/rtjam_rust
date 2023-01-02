use rtjam_rust::{
    config,
    player_list::{get_micro_time, PlayerList},
};
use std::{env, net::UdpSocket};

use rtjam_rust::jam_packet::JamMessage;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(args);

    let mut config = config::Config::build().unwrap();
    println!("filename is : {}", config.get_filename());
    config.load_from_file().unwrap();
    config.dump();

    // So let's create a UDP socket and listen for shit
    let sock = UdpSocket::bind("0.0.0.0:7891").unwrap();
    let mut players = PlayerList::build();
    let mut msg = JamMessage::build();
    let mut cnt: u64 = 0;

    loop {
        cnt += 1;
        let (amt, src) = sock.recv_from(msg.get_buffer()).unwrap();
        if cnt % 1000 == 0 {
            println!("got {} bytes from {}", amt, src);
            println!("player: {}", players);
        }
        // get a timestamp to use
        let now_time = get_micro_time();
        // update the player list
        players.prune(now_time);
        // check if the packet was good
        if amt <= 0 || !msg.is_valid(amt) || !players.is_allowed(msg.get_client_id()) {
            continue;
        }
        // set the server timestamp
        msg.set_server_time(now_time.try_into().unwrap());
        // Update this player with the current time
        players.update_player(now_time, msg.get_client_id(), src);
        for player in players.get_players() {
            if player.address != src {
                // don't send echo back
                // send the packet
                sock.send_to(&msg.get_buffer()[0..amt], player.address)
                    .unwrap();
            }
        }
    }
}
