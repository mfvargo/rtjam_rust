use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
use std::vec;
use std::fs::{File, OpenOptions};

use clap::Parser;
use log::info;
use rtjam_rust::common::box_error::BoxError;
use rtjam_rust::common::get_micro_time;
use rtjam_rust::common::stream_time_stat::MicroTimer;

// #[derive(Parser, Debug)]
// #[command(version, about, long_about = None)]
// struct Args {
//     /// icecast server to connect to
//     #[arg(short, long)]
//     fifo: String,
// }

const FRAME_TIME: u128 = 2_667;
const FRAME_SIZE: usize = 128;

fn main() -> Result<(), BoxError> {
    // let args = Args::parse();
    let mut now = get_micro_time();

    let mut timer = MicroTimer::new(now, FRAME_TIME);
    let buf = vec![0; FRAME_SIZE * 2];
    let mut output = OpenOptions::new()
        .write(true)
        .append(true)
        .open("/home/mfvargo/ices/RAW")
        .unwrap();
    
    loop {
        now = get_micro_time();
        info!("now: {}", now);
        if timer.expired(now) {
            info!("writing to FIFO");
            output.write(&buf)?;
            timer.reset(now);
        }
        sleep(Duration::from_micros(100));
    }
    // return Ok(());
}