use crate::{
    box_error::BoxError,
    jam_packet::JamMessage,
    player_list::{get_micro_time, PlayerList},
};
use std::{io::ErrorKind, net::UdpSocket, sync::mpsc, time::Duration};

pub fn run(port: u32, audio_tx: mpsc::Sender<serde_json::Value>) -> Result<(), BoxError> {
    // So let's create a UDP socket and listen for shit
    let sock = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    sock.set_read_timeout(Some(Duration::new(1, 0)))?;
    let mut players = PlayerList::build();
    let mut msg = JamMessage::build();
    let mut last_latency_update = get_micro_time();

    loop {
        let res = sock.recv_from(msg.get_buffer());
        // get a timestamp to use
        let now_time = get_micro_time();
        // update the player list
        players.prune(now_time);
        match res {
            Ok(r) => {
                let (amt, src) = r;
                if now_time > (last_latency_update + 1000000) {
                    println!("now: {}, last_update: {}", now_time, last_latency_update);
                    last_latency_update = now_time;
                    audio_tx.send(players.get_latency())?;
                    //     println!("got {} bytes from {}", amt, src);
                    //     println!("player: {}", players);
                    //     println!("msg: {}", msg);
                }
                // check if the packet was good
                if amt <= 0 || !msg.is_valid(amt) || !players.is_allowed(msg.get_client_id()) {
                    continue;
                }
                // println!("rcv: {}", msg);
                // Update this player with the current time
                let mut time_diff: u128 = 5000;
                let packet_time = <u64 as TryInto<u128>>::try_into(msg.get_server_time()).unwrap();
                if now_time > packet_time {
                    time_diff = now_time - packet_time;
                }
                players.update_player(now_time, time_diff, msg.get_client_id(), src);

                // set the server timestamp
                msg.set_server_time(now_time.try_into()?);
                // println!("xmit: {}", msg);

                // Broadcast
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
