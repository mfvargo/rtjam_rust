//! REST API to the rtjam-nation to register and activate sound and broadcast elements
//!
//! Super simple.  elements will obtain a token by using the register function for them.
//! This token is then returned so that elements can create websocket chat room with the
//! same name.  This then allows the "meet me in the middle" protocol used by the U/X
use std::collections::HashMap;

use crate::common::box_error::BoxError;
use json::JsonValue;
use reqwest::blocking::Client;
// use serde::{Deserialize, Serialize};

/// The structure that holds state about the api connection
pub struct JamNationApi {
    url_base: String,
    token: String,
    mac_address: String,
    git_hash: String,
    // pub args: NationArgs,
}

impl JamNationApi {
    /// used to build the api.  The fields are
    ///
    /// - base: url base for the api.  (something like <http://rtjam-nation.com/api/1/>)
    /// - lan_ip: legacy parameter.  Should be deprecasted (TODO)
    /// - mac_address: the mac address of the component. Used to uniquely identify the componet
    /// - git_hash: the current git hash string for the build.  Lets the nation know what software component has
    pub fn new(base: &str, mac_address: &str, git_hash: &str) -> JamNationApi {
        JamNationApi {
            token: String::new(),
            url_base: base.to_string(),
            mac_address: mac_address.to_string(),
            git_hash: git_hash.to_string(),
        }
    }
    /// returns the token for the component once it has registered
    pub fn get_token(&self) -> &str {
        self.token.as_str()
    }
    /// Clear the token.  used if the network fails and reconnects
    pub fn forget_token(&mut self) -> () {
        self.token = "".to_string();
    }
    /// Indicates the component has successfully registered
    pub fn has_token(&self) -> bool {
        !self.token.is_empty()
    }
    fn build_def_args(&self) -> HashMap<&str, String> {
        let mut args = HashMap::new();
        args.insert("token", self.token.clone());
        args.insert("lanIp", "10.0.0.0".to_string());
        args.insert("macAddress", self.mac_address.clone());
        args.insert("gitHash", self.git_hash.clone());
        args
    }
    fn get(&self, pth: &str) -> Result<JsonValue, BoxError> {
        let response = Client::new()
            .get(format!("{}{}", self.url_base, pth))
            .send()?;
        if response.status().is_server_error() {
            // Can't parse json on a server error
            return Ok(JsonValue::new_object());
        }
        Ok(json::parse(response.text()?.as_str())?)
    }
    fn put(&self, pth: &str, args: &HashMap<&str, String>) -> Result<JsonValue, BoxError> {
        let client = Client::new();
        let request_url = format!("{}{}", self.url_base, pth);
        let response = client.put(request_url).json(args).send()?;
        if response.status().is_server_error() {
            // Can't parse json on a server error
            return Ok(JsonValue::new_object());
        }
        Ok(json::parse(response.text()?.as_str())?)
    }
    fn post(&self, pth: &str, args: &HashMap<&str, String>) -> Result<JsonValue, BoxError> {
        let client = Client::new();
        let request_url = format!("{}{}", self.url_base, pth);
        let response = client.post(request_url).json(args).send()?;
        if response.status().is_server_error() {
            // Can't parse json on a server error
            return Ok(JsonValue::new_object());
        }
        Ok(json::parse(response.text()?.as_str())?)
    }
    /// call this to just see if the API endpoint is working.
    pub fn get_status(&self) -> Result<JsonValue, BoxError> {
        Ok(self.get("status")?)
    }
    /// Let the nation know the broadcast component is working
    pub fn broadcast_unit_ping(&self) -> Result<JsonValue, BoxError> {
        let args = self.build_def_args();
        Ok(self.put("broadcastUnit/ping", &args)?)
    }
    /// Tell the rtjam-nation that the broadcast component has a room
    pub fn activate_room(&self, port: u32) -> Result<JsonValue, BoxError> {
        let mut args = self.build_def_args();
        args.insert("port", format!("{}", port));
        Ok(self.post("room", &args)?)
    }
    /// Tell the rtjam-nation the broadcast component exists
    pub fn broadcast_unit_register(&mut self) -> Result<JsonValue, BoxError> {
        let args = self.build_def_args();
        let result = self.post("broadcastUnit", &args)?;
        match result["broadcastUnit"]["token"].as_str() {
            Some(v) => {
                self.token = String::from(v);
            }
            None => {}
        }
        Ok(result)
    }
    /// Tell the rtjam-nation the sound component exists
    pub fn jam_unit_register(&mut self) -> Result<JsonValue, BoxError> {
        let mut args = self.build_def_args();
        args.insert("canTalkOnWebsocket", String::from("true"));
        let result = self.post("jamUnit", &args)?;
        match result["jamUnit"]["token"].as_str() {
            Some(v) => {
                self.token = String::from(v);
            }
            None => {}
        }
        Ok(result)
    }
    /// Tell the nation the sound component is alive
    pub fn jam_unit_ping(&self) -> Result<JsonValue, BoxError> {
        let args = self.build_def_args();
        Ok(self.put("jamUnit/ping", &args)?)
    }
}

#[cfg(test)]
mod test_api {
    use super::*;

    fn build_new_api() -> JamNationApi {
        JamNationApi::new("http://localhost/api/1/", "test:mac", "gitHashString")
    }
    #[test]
    fn get_status() {
        // You should get the status from the server
        let api = build_new_api();
        let status = api.get_status().unwrap();
        assert_eq!(status["name"], "rtjam-nation");
    }
    #[test]
    fn broadcast_unit_register_and_ping() {
        let mut api = build_new_api();
        let reg = api.broadcast_unit_register().unwrap();
        assert_eq!(reg["broadcastUnit"]["token"], api.get_token());
        let ping = api.broadcast_unit_ping().unwrap();
        assert!(!ping["broadcastUnit"]["id"].is_empty());
        let activate = api.activate_room(7891).unwrap();
        println!("activate: {}", activate.pretty(2));
    }
    #[test]
    fn jam_unit_register_and_ping() {
        let mut api = build_new_api();
        let reg = api.jam_unit_register().unwrap();
        assert_eq!(reg["jamUnit"]["token"], api.get_token());
        let ping = api.jam_unit_ping().unwrap();
        assert!(!ping["jamUnit"]["id"].is_empty());
    }
}
