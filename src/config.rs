pub struct Config {
    filename: String,
}

impl Config {
    pub fn build() -> Result<Config, &'static str> {
        let filename = String::from("settings.json");

        Ok(Config { filename })
    }
    pub fn get_filename(&self) -> &String {
        &self.filename
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]

    fn config_build() {
        // You should be able to build a Config object
        let config = Config::build().unwrap();
        println!("filename is : {}", config.get_filename());
        assert_eq!(config.get_filename(), "settings.json");
    }
}
