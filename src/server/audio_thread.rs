use crate::{
    common::{
        box_error::BoxError,
        jam_packet::{JamMessage, JAM_HEADER_SIZE},
        stream_time_stat::MicroTimer,
    },
    server::player_list::{get_micro_time, PlayerList},
};
use std::{io::ErrorKind, net::UdpSocket, sync::mpsc, time::Duration};

use super::player_list::MAX_LOOP_TIME;

pub fn run(port: u32, audio_tx: mpsc::Sender<serde_json::Value>) -> Result<(), BoxError> {
    // So let's create a UDP socket and listen for shit
    let sock = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    sock.set_read_timeout(Some(Duration::new(1, 0)))?;
    let mut players = PlayerList::build();
    let mut msg = JamMessage::build();
    let mut latency_update_timer = MicroTimer::new(get_micro_time(), 2_000_000);

    loop {
        let res = sock.recv_from(msg.get_buffer());
        // get a timestamp to use
        let now_time = get_micro_time();
        // update the player list
        players.prune(now_time);
        match res {
            Ok((amt, src)) => {
                if latency_update_timer.expired(now_time) {
                    latency_update_timer.reset(now_time);
                    audio_tx.send(players.get_latency())?;
                    //     println!("got {} bytes from {}", amt, src);
                    // println!("player: {}", players);
                    //     println!("msg: {}", msg);
                }
                // check if the packet was good
                if amt <= 0 || !msg.is_valid(amt) || !players.is_allowed(msg.get_client_id()) {
                    continue;
                }
                // println!("rcv: {}", msg);
                // Update this player with the current time
                let mut time_diff: u128 = MAX_LOOP_TIME;
                let packet_time = msg.get_server_time() as u128;
                if now_time > packet_time {
                    time_diff = now_time - packet_time;
                }
                players.update_player(now_time, time_diff, msg.get_client_id(), src);

                // set the server timestamp
                msg.set_server_time(now_time as u64);
                // println!("xmit: {}", msg);

                // Broadcast
                for player in players.get_players() {
                    if player.address != src {
                        // don't send echo back
                        // send the packet
                        sock.send_to(&msg.get_buffer()[0..amt], player.address)?;
                    } else {
                        // Send just a header to keep the timer looping around
                        sock.send_to(&msg.get_buffer()[0..JAM_HEADER_SIZE], player.address)?;
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
