use rtjam_rust::common::box_error::BoxError;
use clap::Parser;
use rtjam_rust::server::playback_thread::PlaybackMixer;
use rtjam_rust::common::get_micro_time;
use hound;

/// Convert a stored audio packet file into a wave file
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


const FRAME_TIME: u128 = 2_667;

fn main() -> Result<(), BoxError> {
    let args = Args::parse();

    println!("in_file: {}", args.in_file); 
    println!("out_file: {}", args.out_file);

    // So if we get here, we got the input and output files
    let mut mixer = PlaybackMixer::new();
    let mut now = get_micro_time();
    match mixer.open_stream(&args.in_file, now, 0) {
        Ok(()) => {}
        Err(e) => {dbg!(e);}
    }

    // Create a wave file
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(args.out_file, spec)?;

    let mut looping = true;
    while looping {
        // Advance time on frame
        now += FRAME_TIME;
        match mixer.load_up_till_now(now) {
            Ok(()) => {}
            Err(e) => {
                dbg!(e);
                // Probably was end of file. Close the stream.
                mixer.close_stream();
            }            
        }
        match mixer.get_a_frame() {
            Some(buf) => {
                print!(".");
                for i in 0..buf[0].len() {
                    // Since file is 2 channel float write left then write (interleave)
                    writer.write_sample(buf[0][i])?;
                    writer.write_sample(buf[1][i])?;
                }
           }
            None => {
                looping = false;
                println!("No more data to read");
            }
        }
    }

    return Ok(());
}
