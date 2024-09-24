use clap::Parser;
use rtjam_rust::common::box_error::BoxError;
use rtjam_rust::common::get_micro_time;
use rtjam_rust::common::packet_stream::PacketReader;
use csv::Writer;

/// Read Packet originator and timing and save to a CSV file

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Filename of the raw audio
    #[arg(short, long)]
    in_file: String,

    /// Filename for the output
    #[arg(short, long)]
    out_file: String,
}

/// The Row represents a single packets info
#[derive(serde::Serialize)]
struct Row {
    #[serde(rename = "clientId")]
    client_id: u32,
    timestamp: u64,
    sequence: u32,
}

fn main() -> Result<(), BoxError> {
    let args = Args::parse();
    // println!("in_file: {}", args.in_file); 
    // println!("out_file: {}", args.out_file);
    let now = get_micro_time();
    let mut looping = true;
    let mut stream = PacketReader::new(&args.in_file, now)?;
    let mut wtr = Writer::from_path(args.out_file)?;

    while looping {
        match stream.read_packet() {
            Ok(()) => {
                // a packet was read
                wtr.serialize(Row {
                    client_id: stream.get_packet().get_client_id(),
                    timestamp: stream.get_packet().get_server_time(),
                    sequence: stream.get_packet().get_sequence_num()
                })?;
            }
            Err(e) => {
                looping = false;
                dbg!(e);
            }
        }
    }
    return Ok(());
}