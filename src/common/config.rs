//! Allows configuration stuff to be read from settings.json
//!
//! Only use for this now is to allow me to override some of the
//! configuration values for testing locally instead of in the cloud
use json::JsonValue;
use regex::Regex;
use std::{
    fs::File,
    io::{ErrorKind, Write},
    error::Error,
    fmt,
};
use log::{info, warn};

#[derive(Debug)]
pub struct MissingConfigError {
    key: String,
}

impl fmt::Display for MissingConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Required configuration value '{}' is missing", self.key)
    }
}

impl Error for MissingConfigError {}

pub struct Config {
    filename: String,
    settings: JsonValue,
    defaults: JsonValue,
}

impl Config {
    pub fn build(filename: String, defaults: JsonValue) -> Result<Config, std::io::Error> {
        // Try to open the file to validate filename
        // Validate filename only contains valid characters and ends in .json
        let filename_regex = Regex::new(r"^(?:[/a-zA-Z0-9_\-\.]+)*?[/a-zA-Z0-9_\-\.]+\.json$").unwrap();
        if !filename_regex.is_match(&filename) {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!("Invalid filename '{}' - must contain only letters, numbers, underscore, dash, dot and end in .json", filename)
            ));
        }

        let mut config = Config {
            filename,
            settings: json::object! {},
            defaults,
        };
        
        if let Err(err) = config.load_from_file() {
            warn!("Using default settings: {}", err);
        }
        
        Ok(config)
    }

    fn load_from_file(&mut self) -> std::io::Result<()> {
        match std::fs::read_to_string(&self.filename) {
            Ok(raw_data) => {
                match json::parse(&raw_data) {
                    Ok(parsed) => {
                        self.settings.clone_from(&parsed);
                        info!("Loaded settings from {}: {}", self.filename, self.settings.pretty(2));
                        Ok(())
                    },
                    Err(err) => {
                        warn!("Failed to parse config file {}: {}", self.filename, err);
                        Ok(())
                    }
                }
            }
            Err(err) => Err(err)
        }
    }

    pub fn get_str_value(&self, key: &str, default: Option<String>) -> Result<String, MissingConfigError> {
        // First check settings
        if let Some(val) = self.settings[key].as_str() {
            return Ok(val.to_string());
        }

        // If explicit default is provided, use it
        if let Some(def) = default {
            return Ok(def);
        }

        // Otherwise check defaults
        if let Some(val) = self.defaults[key].as_str() {
            return Ok(val.to_string());
        }

        // If no value found anywhere, return error
        Err(MissingConfigError { key: key.to_string() })
    }

    pub fn get_bool_value(&self, key: &str, default: Option<bool>) -> Result<bool, MissingConfigError> {
        // First check settings
        if let Some(val) = self.settings[key].as_bool() {
            return Ok(val);
        }

        // If explicit default is provided, use it
        if let Some(def) = default {
            return Ok(def);
        }

        // Otherwise check defaults
        if let Some(val) = self.defaults[key].as_bool() {
            return Ok(val);
        }

        // If no value found anywhere, return error
        Err(MissingConfigError { key: key.to_string() })
    }

    pub fn get_u32_value(&self, key: &str, default: Option<u32>) -> Result<u32, MissingConfigError> {
        // First check settings
        if let Some(val) = self.settings[key].as_u32() {
            return Ok(val);
        }

        // If explicit default is provided, use it 
        if let Some(def) = default {
            return Ok(def);
        }

        // Otherwise check defaults
        if let Some(val) = self.defaults[key].as_u32() {
            return Ok(val);
        }

        // If no value found anywhere, return error
        Err(MissingConfigError { key: key.to_string() })
    }

    pub fn set_value(&mut self, key: &str, val: impl Into<JsonValue>) -> Result<(), String> {
        let json_val = val.into();
        match json_val {
            JsonValue::Short(_) | JsonValue::String(_) | JsonValue::Boolean(_) | JsonValue::Number(_) => {
                self.settings[key] = json_val;
                Ok(())
            },
            _ => Err(format!("Unsupported value type for key: {}", key)),
        }
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
mod config_tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::path::PathBuf;

    /// A totally over-engineered test context that efficiently creates a one time shared temporary directory and config file
    /// and deletes them when the test suite is done.
    struct TestContext {
        config_path: PathBuf,
        file_values: JsonValue,
        defaults: JsonValue,
    }

    static TEST_CONTEXT: Lazy<TestContext> = Lazy::new(|| {
        let file_content = r#"{
            "file_string": "I live in a fishbowl",
            "file_bool": false,
            "file_u32": 2112
        }"#;
        
        let config_path = std::env::temp_dir().join("test_config.json");
        let file_values = json::parse(file_content).expect("Failed to parse file content");
        let defaults = json::object! {
            "file_string": "default_name",
            "file_bool": true,
            "file_u32": 42,
            "default_string": "default_only",
        };

        // Create a temporary config file
        std::fs::write(&config_path, file_content).expect("Failed to write temporary config file");

        // Return a context for referencing expected values
        TestContext {
            config_path,
            file_values,
            defaults,
        }
    });

    // Helper function to create a Config instance with the shared config file
    fn create_config_with_test_file() -> Config {
        Config::build(
            TEST_CONTEXT.config_path.to_str().unwrap().to_string(),
            TEST_CONTEXT.defaults.clone()
        ).expect("Failed to build config")
    }

    // Helper function to create a Config instance with the shared config file
    fn create_config_with_filename(filename: &str) -> Config {
        Config::build(
            filename.to_string(),
            TEST_CONTEXT.defaults.clone()
        ).expect("Failed to build config")
    }

    fn set_config_value_or_boom(config: &mut Config, key: &str, val: impl Into<JsonValue>) -> Result<(), String> {
        let set_result = config.set_value(key, val);
        assert_eq!(set_result.is_ok(), true);
        Ok(())
    }
    
    #[test]
    fn should_build() {
        let config: Config = create_config_with_test_file();
        assert_eq!(config.filename, TEST_CONTEXT.config_path.to_str().unwrap());
    }

    #[test]
    fn should_build_with_any_valid_name() {
        // you should be able to build a config object from a valid file name, if it doesn't exist
        let config: Config = create_config_with_filename("I_see_dead_people.json");
        // Confirm that a config instance is returned
        assert_eq!(config.filename, "I_see_dead_people.json");
    }

    #[test]
    fn should_return_file_values() {
        let config: Config = create_config_with_test_file();
        assert_eq!(config.get_str_value("file_string", None).unwrap(), TEST_CONTEXT.file_values["file_string"].as_str().unwrap());
        assert_eq!(config.get_bool_value("file_bool", None).unwrap(), TEST_CONTEXT.file_values["file_bool"].as_bool().unwrap());
        assert_eq!(config.get_u32_value("file_u32", None).unwrap(), TEST_CONTEXT.file_values["file_u32"].as_u32().unwrap());
    }

    #[test]
    fn should_get_defaults_with_no_file() {
        let config: Config = create_config_with_filename("I_see_dead_people.json");
        // Confirm that an instance of each type of value can be retrieved from the config object
        assert_eq!(config.get_str_value("file_string", None).unwrap(), TEST_CONTEXT.defaults["file_string"].as_str().unwrap());
        assert_eq!(config.get_bool_value("file_bool", None).unwrap(), TEST_CONTEXT.defaults["file_bool"].as_bool().unwrap());
        assert_eq!(config.get_u32_value("file_u32", None).unwrap(), TEST_CONTEXT.defaults["file_u32"].as_u32().unwrap());
        assert_eq!(config.get_str_value("default_string", None).unwrap(), TEST_CONTEXT.defaults["default_string"].as_str().unwrap());
    }

    #[test]
    fn should_error_with_invalid_name() {
        // you should not be able to build a config object from an invalid file name
        let filename = "I'm_;,`all_{jacked}_up";
        let boom: Result<Config, std::io::Error> = Config::build(filename.to_string(), TEST_CONTEXT.defaults.clone());
        // Print the type of boom for debugging
        println!("Type of boom: {}", std::any::type_name_of_val(&boom));
        match boom {
            Ok(_) => assert!(false, "Expected error for invalid filename"),
            Err(e) => assert_eq!(e.kind(), ErrorKind::InvalidInput)
        }
    }

    #[test]
    fn should_dump() {
        // you should be able to dump the config object
        let config: Config = create_config_with_test_file();
        assert_eq!(config.dump(), ());
    }

    #[test]
    fn get_str_value_config_default() {
        // You should be able to get config objects default value for a string
        let config: Config = create_config_with_filename("no_file.json");
        assert_eq!(config.get_str_value("file_string", None).unwrap(), TEST_CONTEXT.defaults["file_string"].as_str().unwrap());
    }

    #[test]
    fn get_str_value_explicit_set() {
        // You should be able to get a set string value that overrides the config default
        let mut config: Config = create_config_with_filename("no_file.json");
        let _ = set_config_value_or_boom(&mut config, "name", "new value");
        assert_eq!(config.get_str_value("name", None).unwrap(), "new value");
    }

    #[test]
    fn get_str_value_with_explicit_default() {
        // You should be able to get a string value with an explicit default in the get fn
        let config: Config = create_config_with_filename("no_file.json");
        assert_eq!(config.get_str_value("i_dont_exist", Some("default value".to_string())).unwrap(), "default value");
    }

    #[test]
    fn get_str_value_error_on_missing_key() {
        let config: Config = create_config_with_filename("no_file.json");
        let boom: Result<String, MissingConfigError> = config.get_str_value("i_dont_exist", None);
        assert_eq!(boom.is_err(), true);
        // assert the type is MissingConfigError
        assert_eq!(boom.err().unwrap().to_string(), "Required configuration value 'i_dont_exist' is missing");
    }

    #[test]
    fn get_bool_value_config_default() {
        // You should be able to get config objects default value for a boolean
        let config: Config = create_config_with_filename("no_file.json");
        assert_eq!(config.get_bool_value("file_bool", None).unwrap(), TEST_CONTEXT.defaults["file_bool"].as_bool().unwrap());
    }

    #[test]
    fn get_bool_value_explicit_set() {
        // You should be able to get a set boolean value that overrides the config default
        let mut config: Config = create_config_with_filename("no_file.json");
        let _ = set_config_value_or_boom(&mut config, "enabled", false);
        assert_eq!(config.get_bool_value("enabled", None).unwrap(), false);
    }

    #[test]
    fn get_bool_value_with_explicit_default() {
        // You should be able to get a boolean value with an explicit default in the get fn
        let config: Config = create_config_with_filename("no_file.json");
        assert_eq!(config.get_bool_value("i_dont_exist", Some(false)).unwrap(), false);
    }

    #[test]
    fn get_bool_value_error_on_missing_key() {
        let config: Config = create_config_with_filename("no_file.json");
        let boom: Result<bool, MissingConfigError> = config.get_bool_value("i_dont_exist", None);
        assert_eq!(boom.is_err(), true);
        // assert the type is MissingConfigError
        assert_eq!(boom.err().unwrap().to_string(), "Required configuration value 'i_dont_exist' is missing");
    }

    #[test]
    fn get_u32_value_config_default() {
        // You should be able to get config objects default value for a u32
        let config: Config = create_config_with_filename("no_file.json");
        assert_eq!(config.get_u32_value("file_u32", None).unwrap(), TEST_CONTEXT.defaults["file_u32"].as_u32().unwrap());
    }

    #[test]
    fn get_u32_value_explicit_set() {
        // You should be able to get a set u32 value that overrides the config default
        let mut config: Config = create_config_with_filename("no_file.json");
        let _ = set_config_value_or_boom(&mut config, "count", 100);
        assert_eq!(config.get_u32_value("count", None).unwrap(), 100);
    }

    #[test]
    fn get_u32_value_with_explicit_default() {
        // You should be able to get a u32 value with an explicit default in the get fn
        let config: Config = create_config_with_filename("no_file.json");
        assert_eq!(config.get_u32_value("i_dont_exist", Some(99)).unwrap(), 99);
    }

    #[test]
    fn get_u32_value_error_on_missing_key() {
        let config: Config = create_config_with_filename("no_file.json");
        let boom: Result<u32, MissingConfigError> = config.get_u32_value("i_dont_exist", None);
        assert_eq!(boom.is_err(), true);
        // assert the type is MissingConfigError
        assert_eq!(boom.err().unwrap().to_string(), "Required configuration value 'i_dont_exist' is missing");
    }

    #[test]
    fn set_value_str() {
        // You should be able to set a value on a key
        let mut config: Config = create_config_with_filename("no_file.json");
        let _ = set_config_value_or_boom(&mut config, "lastname", "kajikami");
        let lastname: Result::<String, MissingConfigError> = config.get_str_value("lastname", None);
        assert_eq!(lastname.unwrap(), "kajikami");
    }

    #[test]
    fn set_value_with_unsupported_type() {
        // You should not be able to set a value with an unsupported type (e.g., array)
        let mut config: Config = create_config_with_filename("no_file.json");
        let set_result = config.set_value("unsupported", json::array!["value1", "value2"]);
        assert_eq!(set_result.is_err(), true);
        assert_eq!(set_result.err().unwrap(), "Unsupported value type for key: unsupported");
    }

    #[test]
    fn save_settings() {
        // You should be able to flush the settings to the file
        let mut config: Config = create_config_with_filename("settings.json");
        let _ = set_config_value_or_boom(&mut config, "foobar", "as Usual");
        let result = config.save_settings();
        assert_eq!(result.unwrap(), true);
        config.dump();
    }
}
