use clap::{Parser, command};
use rtjam_rust::{common::box_error::BoxError, sound::client, utils::get_git_hash};
use std::process::exit;
use env_logger::Builder;
use log::{info, LevelFilter};

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

    // TODO: Move this to common and share as a simple fn for all executables
    // Initialize the logger, configuring it to write to stdout
    Builder::new()
        .filter_level(LevelFilter::Debug)
        .target(env_logger::Target::Stdout)
        .init();
   
    // note: add error checking yourself.
    let git_hash = get_git_hash().to_string();
    let args = Args::parse();
    if args.version {
        // Used to directly query for version, so no fancy formatting here. Sorry logger!
        println!("{}", git_hash);
        exit(0);
    }
    let driver = if args.alsa { "ALSA" } else { "Jack" };
    info!("Starting rtjam_sound with in: {}, out: {}, using {}", args.in_dev, args.out_dev, driver);
    client::run(git_hash, args.alsa, args.in_dev, args.out_dev)?;
    Ok(())
}
