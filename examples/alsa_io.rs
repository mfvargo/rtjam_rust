use std::thread;

use rtjam_rust::{common::box_error::BoxError, sound::alsa_thread};

fn main() -> Result<(), BoxError> {
    // note: add error checking yourself.

    let alsa_handle = thread::spawn(move || {
        let _res = alsa_thread::run("hw:CODEC");
    });

    let res = alsa_handle.join();
    dbg!(res);
    Ok(())
}
