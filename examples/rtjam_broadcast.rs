use rtjam_rust::config;
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
    let sock = UdpSocket::bind("127.0.0.1:7891").unwrap();
    let mut msg = JamMessage::build().unwrap();

    loop {
        let (amt, src) = sock.recv_from(msg.get_buffer()).unwrap();
        println!("got {} bytes from {}", amt, src);
    }
}
