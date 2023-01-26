use crate::box_error::BoxError;

pub fn run(_git_hash: &str) -> Result<(), BoxError> {
    // This is the entry rtjam client
    println!("rtjam client");
    Ok(())
}
