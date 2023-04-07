//! listen for packets from sound components and multicast them to people in the room
//!
//! The socket read is non-blocking.
use crate::{
    common::{
        box_error::BoxError,
        get_micro_time,
        jam_packet::{JamMessage, JAM_HEADER_SIZE},
        player::MAX_LOOP_TIME,
        sock_with_tos,
        stream_time_stat::MicroTimer,
        websock_message::WebsockMessage,
    },
    server::player_list::PlayerList,
};
use serde_json::json;
use std::{io::ErrorKind, sync::mpsc, time::Duration};

pub fn run(port: u32, audio_tx: mpsc::Sender<WebsockMessage>, token: &str) -> Result<(), BoxError> {
    // So let's create a UDP socket and listen for shit
    let sock = sock_with_tos::new(port);
    sock.set_read_timeout(Some(Duration::new(1, 0)))?;
    let mut players = PlayerList::new();
    let mut msg = JamMessage::new();
    let mut latency_update_timer = MicroTimer::new(get_micro_time(), 2_000_000);

    loop {
        let res = sock.recv_from(msg.get_buffer());
        // get a timestamp to use
        let now_time = get_micro_time();
        // update the player list
        players.prune(now_time);
        if latency_update_timer.expired(now_time) {
            latency_update_timer.reset(now_time);
            audio_tx.send(WebsockMessage::Chat(players.get_latency()))?;
            //     println!("got {} bytes from {}", amt, src);
            println!("player: {}", players);
            //     println!("msg: {}", msg);
            while players.stat_queue.len() > 0 {
                if let Some(stats) = players.stat_queue.pop() {
                    audio_tx.send(WebsockMessage::API(
                        "packetStatCreate".to_string(),
                        json!({ "roomToken": token, "stats": stats }),
                    ))?;
                }
            }
            // audio_tx.send(WebsockMessage::API("status".to_string(), json!({})))?;
        }
        match res {
            Ok((amt, src)) => {
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
                players.update_player(
                    now_time,
                    time_diff,
                    msg.get_client_id(),
                    src,
                    msg.get_sequence_num(),
                );

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
                ErrorKind::TimedOut => {}
                other_error => {
                    panic!("my socket went nuts! {}", other_error);
                }
            },
        }
    }
}
