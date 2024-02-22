//! Allows configuration stuff to be read from settings.json
//!
//! Only use for this now is to allow me to override some of the
//! configuration values for testing locally instead of in the cloud
use json::JsonValue;
use std::{
    fs::File,
    io::{ErrorKind, Write},
};
use log::{info};

pub struct Config {
    filename: String,
    settings: JsonValue,
}

impl Config {
    pub fn build(filename: String) -> Config {
        // TODO: Validate that filename is a valid file name, ends in .json, and that the path exists
        let open_test = std::fs::OpenOptions::new()
            .open(filename.as_str());
        match open_test {
            Ok(_) => {
                Config {
                    filename: filename,
                    settings: json::object! {},
                }
            }
            Err(err) => {
                panic!("Cannot open settings file: {}", err);
            }
        }
    }

    pub fn get_filename(&self) -> &str {
        &self.filename
    }
    pub fn load_from_file(&mut self) -> std::io::Result<bool> {
        match std::fs::read_to_string(&self.filename) {
            Ok(raw_data) => {
                // we were able to read the file
                let parsed = json::parse(&raw_data).unwrap();
                self.settings.clone_from(&parsed);
                info!("settings: {}", self.settings.pretty(2));
                Ok(true)
            }
            Err(_) => {
                // call save settings to create a new file
                self.save_settings()
            }
        }
    }

    pub fn get_value<'a>(&'a mut self, key: &str, def_value: &'a str) -> &str {
        let val = self.settings[key].as_str();
        match val {
            None => def_value,
            Some(i) => i,
        }
    }
    pub fn get_u32_value(&self, key: &str, def_value: u32) -> u32 {
        let val = self.settings[key].as_u32();
        match val {
            None => def_value,
            Some(i) => i,
        }
    }
    pub fn get_bool_value(&self, key: &str, def_value: bool) -> bool {
        // expects JSON value as type bool (unquoted), any string and bad things happen
        let val = self.settings[key].as_bool();
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

    pub fn save_settings(&self) -> std::io::Result<bool> {
        let file_open_result = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.filename.as_str());
        match file_open_result {
            Ok(mut f) => self.flush_to_file(&mut f),
            Err(error) => {
                // File open failed.  See if we need to create it
                match error.kind() {
                    ErrorKind::NotFound => {
                        // no file, create one
                        let mut f = std::fs::File::create(self.filename.as_str())?;
                        self.flush_to_file(&mut f)
                    }
                    other_error => {
                        panic!("Cannot create settings file: {}", other_error);
                    }
                }
            }
        }
    }
    fn flush_to_file(&self, f: &mut File) -> std::io::Result<bool> {
        f.write_all(self.settings.pretty(2).as_bytes())?;
        f.sync_all()?;
        Ok(true)
    }
}
#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_build_with_any_valid_name() {
        // you should be able to build a config object from a valid file name, if it doesn't exist
        let filename = String::from("I_see_dead_people.json");
        let config = Config::build(filename.clone());
        // Confirm that a config instance is returned
        assert_eq!(config.filename, filename);
    }

    #[test]
    #[should_panic]
    fn should_panic_with_invalid_name() {
        // you should not be able to build a config object from an invalid file name
        let filename = String::from("I'm_\\all_{jacked}_up");
        let _config = Config::build(filename);
    }

    #[test]
    fn should_dump() {
        // you should be able to dump the config object
        let config: Config = Config::build(String::from("settings.json"));
        assert_eq!(config.dump(), ());
    }

    #[test]
    fn load_from_file() {
        // The configuration should serialize itself to JSON
        let mut config: Config = Config::build(String::from("settings.json"));
        assert_eq!(config.load_from_file().unwrap(), true);
    }

    #[test]
    fn get_value_default() {
        // You should be able to get a value with a default
        let mut config: Config = Config::build(String::from("settings.json"));
        let bob = config.get_value("bob", "bob");
        assert_eq!(bob, "bob");
    }
    
    #[test]
    fn get_bool_value_default() {
        // You should be able to get a bool value with a default
        let config: Config = Config::build(String::from("settings.json"));
        let value = config.get_bool_value("i_dont_exist", false);
        assert_eq!(value, false);
    }

    #[test]
    fn get_bool_value() {
        // You should be able to get a bool value
        let config = Config{
            filename: String::from("no_file"),
            settings: json::object! {
                "the_truth": true
            },
        };
        let value = config.get_bool_value("the_truth", false);
        assert_eq!(value, true);
    }

    #[test]
    fn get_bool_value_invalid() {
        // A non-bool JSON type should be handled gracefully, with a default value and log a warning
        let config = Config{
            filename: String::from("no_file"),
            settings: json::object! {
                "should_be_bool": "not a bool"
            },
        };
        let value = config.get_bool_value("should_be_bool", true);
        assert_eq!(value, true);
        // TODO: validate that a warning was logged to console
        // use the captured output crate for this? 
    }
   
    // TODO: add tests for the other get_<type>_value functions

    #[test]
    fn set_value() {
        // You should be able to set a value on a key
        let mut config: Config = Config::build(String::from("settings.json"));
        config.set_value("lastname", "kajikami");
        let lastname = config.get_value("lastname", "smith");
        assert_eq!(lastname, "kajikami");
    }
    #[test]
    fn save_settings() {
        // You should be able to flush the settings to the file
        let mut config: Config = Config::build(String::from("settings.json"));
        config.load_from_file().unwrap();
        config.set_value("foobar", "as Usual");
        let result = config.save_settings();
        assert_eq!(result.unwrap(), true);
        config.dump();
    }
}
