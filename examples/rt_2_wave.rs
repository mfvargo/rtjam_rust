use rtjam_rust::common::box_error::BoxError;
use clap::Parser;

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
fn main() -> Result<(), BoxError> {
    let args = Args::parse();

    println!("in_file: {}", args.in_file); 
    println!("out_file: {}", args.out_file);

    // So if we get here, we got the input and output files
    

    return Ok(());
}
