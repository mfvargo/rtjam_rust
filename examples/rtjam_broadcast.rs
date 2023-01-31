use rtjam_rust::{common::box_error::BoxError, server::broadcast_server, utils::get_git_hash};

fn main() -> Result<(), BoxError> {
    // note: add error checking yourself.
    let git_hash = get_git_hash();
    println!("GIT_HASH: {}", git_hash);
    broadcast_server::run(git_hash.as_str())?;
    Ok(())
}
