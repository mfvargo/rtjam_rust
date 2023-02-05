use rtjam_rust::{common::box_error::BoxError, server::broadcast_server, utils::get_git_hash};
use std::{env, process::exit};

fn main() -> Result<(), BoxError> {
    // note: add error checking yourself.
    let git_hash = get_git_hash();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        println!("{}", git_hash);
        exit(0);
    }
    broadcast_server::run(git_hash.as_str())?;
    Ok(())
}
