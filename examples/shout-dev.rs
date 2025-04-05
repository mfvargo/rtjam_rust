use std::thread::sleep;
use std::time::Duration;

use clap::Parser;
use rtjam_rust::common::box_error::BoxError;
use rtjam_rust::common::get_micro_time;
use rtjam_rust::common::jam_packet::JamMessage;
use rtjam_rust::common::stream_time_stat::MicroTimer;
use rtjam_rust::sound::jitter_buffer::JitterBuffer;
use shout::ShoutAudioInfo;
use shout::ShoutConnBuilder;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// icecast server to connect to
    #[arg(short, long)]
    server: String,

    /// port number on icecast server
    #[arg(short, long)]
    port: u16,
}

const FRAME_TIME: u128 = 2_667;

fn main() -> Result<(), BoxError> {
    let args = Args::parse();
    let mut now = get_micro_time();
    let mut looping = true;
    let mut pback_timer = MicroTimer::new(get_micro_time(), FRAME_TIME);

    let conn = ShoutConnBuilder::new()
        .host(args.server)
        .port(args.port)
        .user(String::from("source"))
        .password(String::from("G0bzdog1"))
        .mount(String::from("/test.mp3"))
        .protocol(shout::ShoutProtocol::HTTP)
        .format(shout::ShoutFormat::MP3)
        .add_audio_info(ShoutAudioInfo::SampleRate("48000".to_string()))
        .add_audio_info(ShoutAudioInfo::Channels("1".to_string()))
        .build().unwrap();

    println!("Connected to server");

    let mut jitter_buffer = JitterBuffer::new();
    let mut msg = JamMessage::new();

    while looping {
        // now = get_micro_time();
        // if pback_timer.expired(now) {
        //     pback_timer.advance(FRAME_TIME);
        //     let buf = jitter_buffer.get(128, 0.0);
        //     msg.encode_audio(&buf, &buf);
        //     conn.send(msg.get_audio_space(128 * 2)).unwrap();
        // }
        // sleep(Duration::new(0, 100_000));
        sleep(Duration::new(1000, 100_000));
        looping = false;
    }
    return Ok(());
}