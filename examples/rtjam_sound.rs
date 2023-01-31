use rtjam_rust::{common::box_error::BoxError, sound::client, utils::get_git_hash};

fn main() -> Result<(), BoxError> {
    // note: add error checking yourself.
    let git_hash = get_git_hash();
    println!("GIT_HASH: {}", git_hash);
    client::run(git_hash.as_str())?;
    Ok(())
}
