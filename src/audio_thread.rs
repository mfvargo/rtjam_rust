use crate::{
    box_error::BoxError,
    jam_packet::JamMessage,
    player_list::{get_micro_time, PlayerList},
};
use json::JsonValue;
use std::{io::ErrorKind, net::UdpSocket, sync::mpsc, time::Duration};

pub fn run(port: u32, audio_tx: mpsc::Sender<JsonValue>) -> Result<(), BoxError> {
    // So let's create a UDP socket and listen for shit
    let sock = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    sock.set_read_timeout(Some(Duration::new(1, 0)))?;
    let mut players = PlayerList::build();
    let mut msg = JamMessage::build();
    let mut cnt: u64 = 0;

    loop {
        cnt += 1;
        let res = sock.recv_from(msg.get_buffer());
        // get a timestamp to use
        let now_time = get_micro_time();
        // update the player list
        players.prune(now_time);
        match res {
            Ok(r) => {
                let (amt, src) = r;
                if cnt % 1000 == 0 {
                    println!("got {} bytes from {}", amt, src);
                    println!("player: {}", players);
                    audio_tx.send(players.as_json())?;
                }
                // check if the packet was good
                if amt <= 0 || !msg.is_valid(amt) || !players.is_allowed(msg.get_client_id()) {
                    continue;
                }
                // set the server timestamp
                msg.set_server_time(now_time.try_into()?);
                // Update this player with the current time
                players.update_player(now_time, msg.get_client_id(), src);
                for player in players.get_players() {
                    if player.address != src {
                        // don't send echo back
                        // send the packet
                        sock.send_to(&msg.get_buffer()[0..amt], player.address)?;
                    }
                }
            }
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => {}
                other_error => {
                    panic!("my socket went nuts! {}", other_error);
                }
            },
        }
    }
}
