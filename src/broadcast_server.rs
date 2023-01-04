use crate::{
    box_error::BoxError,
    config::Config,
    jam_nation_api::JamNationApi,
    jam_packet::JamMessage,
    player_list::{get_micro_time, PlayerList},
};
use std::{
    io::ErrorKind,
    net::UdpSocket,
    thread::{self, sleep},
    time::Duration,
};

pub fn run(git_hash: &str) -> Result<(), BoxError> {
    // This is the entry point for the broadcast server

    // Create a thread to host the room
    let room_handle = thread::spawn(|| {
        let _res = audio_thread(7891);
    });
    // Now the main thread will run the broadcast keepalive code
    broadcast_keepalive(git_hash)?;
    let _res = room_handle.join();
    Ok(())
}

fn audio_thread(port: u32) -> Result<(), BoxError> {
    // So let's create a UDP socket and listen for shit
    let sock = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    let res = sock.set_read_timeout(Some(Duration::new(1, 0)))?;
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

fn broadcast_keepalive(git_hash: &str) -> Result<(), BoxError> {
    // Get the Config Object
    let mut config = Config::build()?;
    config.load_from_file()?;
    let api_url = config.get_value(
        "rtjam-nation",
        "http://rtjam-nation.basscleftech.com/api/1/",
    );
    println!("api endpoint: {}", api_url);
    // create the api
    let mut api = JamNationApi::new(api_url, "10.10.10.10", "test:mac", "gitHashString");

    loop {
        while api.has_token() == true {
            // While in this loop, we are going to ping every 10 seconds
            let ping = api.broadcast_unit_ping()?;
            if !ping["error"].is_empty() {
                // Error in the ping.  better re-register
                println!("ping: {}", ping.pretty(2));
                api.forget_token();
            } else {
                // Successful ping.. Sleep for 10
                println!("ping!");
                sleep(Duration::new(10, 0));
            }
        }
        if !api.has_token() {
            // We need to register the server
            let reg = api.broadcast_unit_register()?;
            println!("register: {}", reg.pretty(2));
        }
        // This is the timer between registration attempts
        sleep(Duration::new(2, 0));
    }
}
