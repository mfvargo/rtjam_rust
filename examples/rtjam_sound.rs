use clap::{Parser, command};
use rtjam_rust::{common::box_error::BoxError, sound::client, utils::get_git_hash};
use std::process::exit;

#[derive(Parser)]
#[command(version, about, long_about = None, disable_version_flag = true)]
struct Args {
    /// input alsa device
    #[arg(short, long, default_value = "hw:SigmaI2SCodec,1")]
    in_dev: String,

    /// output alsa device
    #[arg(short, long, default_value = "hw:SigmaI2SCodec,0")]
    out_dev: String,

    #[arg(short, long, default_value_t = false)]
    version: bool,

    #[arg(short, long, default_value_t = false)]
    alsa: bool,
}


fn main() -> Result<(), BoxError> {

    // Turn on the logger
    env_logger::init();
    
    // note: add error checking yourself.
    let git_hash = get_git_hash().to_string();
    let args = Args::parse();
    if args.version {
        println!("{}", git_hash);
        exit(0);
    }
    client::run(git_hash, args.alsa, args.in_dev, args.out_dev)?;
    Ok(())
}
