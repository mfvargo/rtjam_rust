use std::fs::File;
use std::io::{self, Read};
use std::io::{ErrorKind, Write};

use json::JsonValue;
pub struct Config {
    filename: String,
    settings: JsonValue,
}

impl Config {
    pub fn build() -> Result<Config, &'static str> {
        Ok(Config {
            filename: String::from("settings.json"),
            settings: json::object! {},
        })
    }
    pub fn get_filename(&self) -> &str {
        &self.filename
    }
    pub fn load_from_file(&mut self) -> Result<bool, io::Error> {
        let setting_file_result = File::open(&self.filename);
        let mut setting_file = match setting_file_result {
            Ok(file) => file,
            Err(error) => match error.kind() {
                ErrorKind::NotFound => match File::create(&self.filename) {
                    Ok(fc) => fc,
                    Err(e) => panic!("Cannot create settings.json: {:?}", e),
                },
                other_error => {
                    panic!("Problem opening the file: {:?}", other_error);
                }
            },
        };

        let mut raw_data = String::new();

        setting_file.read_to_string(&mut raw_data)?;

        let parsed = json::parse(&raw_data).unwrap();
        self.settings.clone_from(&parsed);

        Ok(true)
    }

    pub fn get_value<'a>(&'a mut self, key: &str, def_value: &'a str) -> &str {
        let val = self.settings[key].as_str();
        match val {
            None => def_value,
            Some(i) => i,
        }
    }

    pub fn set_value(&mut self, key: &str, val: &str) -> () {
        self.settings[key] = val.into();
    }

    pub fn dump(&self) {
        println!("config dump: {}", self.settings.pretty(2));
    }

    pub fn save_settings(&self) -> std::io::Result<()> {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.filename.as_str())?;
        f.write_all(self.settings.pretty(2).as_bytes())?;
        f.sync_all()?;
        Ok(())
    }
}
#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_dump() {
        // you should be able to dump the config object
        let config = Config::build().unwrap();
        assert_eq!(config.dump(), ());
    }

    #[test]

    fn config_build() {
        // You should be able to build a Config object
        let config = Config::build().unwrap();
        println!("filename is : {}", config.get_filename());
        assert_eq!(config.get_filename(), "settings.json");
    }

    #[test]
    fn load_from_file() {
        // The configuration should serialize itself to JSON
        let mut config = Config::build().unwrap();
        assert_eq!(config.load_from_file().unwrap(), true);
    }

    #[test]
    fn get_value_default() {
        // You should be able to get a value with a default
        let mut config = Config::build().unwrap();
        let bob = config.get_value("bob", "bob");
        assert_eq!(bob, "bob");
    }
    #[test]
    fn set_value() {
        // You should be able to set a value on a key
        let mut config = Config::build().unwrap();
        config.set_value("lastname", "kajikami");
        let lastname = config.get_value("lastname", "smith");
        assert_eq!(lastname, "kajikami");
    }
    #[test]
    fn save_config() {
        // You should be able to flush the settings to the file
        let mut config = Config::build().unwrap();
        config.load_from_file().unwrap();
        config.set_value("foobar", "as Usual");
        let result = config.save_settings();
        assert_eq!(result.unwrap(), ());
        config.dump();
    }
}
