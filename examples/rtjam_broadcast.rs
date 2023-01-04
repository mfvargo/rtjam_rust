use rtjam_rust::{box_error::BoxError, broadcast_server};
use std::process::Command;

fn main() -> Result<(), BoxError> {
    // note: add error checking yourself.
    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output()?;
    let git_hash = String::from_utf8(output.stdout)?;
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    broadcast_server::run(git_hash.as_str())?;
    Ok(())
}
