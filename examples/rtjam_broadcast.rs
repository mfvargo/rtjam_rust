use rtjam_rust::config;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(args);

    let mut config = config::Config::build().unwrap();
    println!("filename is : {}", config.get_filename());
    config.load_from_file().unwrap();
    config.dump();
}
