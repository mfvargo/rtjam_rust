use rtjam_rust::{box_error::BoxError, rtjam_client};
use std::process::Command;

fn main() -> Result<(), BoxError> {
    // note: add error checking yourself.
    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output()?;
    let git_hash = String::from_utf8(output.stdout)?;
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    rtjam_client::run(git_hash.as_str())?;
    Ok(())
}
