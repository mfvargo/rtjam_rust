use clap::Parser;
use rtjam_rust::{common::box_error::BoxError, sound::client, utils::get_git_hash};
use std::process::exit;


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// input alsa device
    #[arg(short, long, default_value = "hw:CODEC")]
    in_dev: String,

    /// output alsa device
    #[arg(short, long, default_value = "hw:CODEC")]
    out_dev: String,

    #[arg(short, long, default_value_t = false)]
    version: bool,
}

fn main() -> Result<(), BoxError> {
    // note: add error checking yourself.
    let git_hash = get_git_hash();
    let args = Args::parse();
    if args.version {
        println!("{}", git_hash);
        exit(0);
    }
    client::run(git_hash.as_str(), args.in_dev, args.out_dev)?;
    Ok(())
}
