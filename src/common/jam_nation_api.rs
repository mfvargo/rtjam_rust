use std::collections::HashMap;

use crate::common::box_error::BoxError;
use json::JsonValue;
use reqwest::blocking::Client;
// use serde::{Deserialize, Serialize};

pub struct JamNationApi {
    url_base: String,
    token: String,
    lan_ip: String,
    mac_address: String,
    git_hash: String,
    // pub args: NationArgs,
}

impl JamNationApi {
    pub fn new(base: &str, lan_ip: &str, mac_address: &str, git_hash: &str) -> JamNationApi {
        JamNationApi {
            token: String::new(),
            url_base: base.to_string(),
            lan_ip: lan_ip.to_string(),
            mac_address: mac_address.to_string(),
            git_hash: git_hash.to_string(),
        }
    }
    pub fn get_token(&self) -> &str {
        self.token.as_str()
    }
    pub fn forget_token(&mut self) -> () {
        self.token = "".to_string();
    }
    pub fn has_token(&self) -> bool {
        !self.token.is_empty()
    }
    pub fn build_def_args(&self) -> HashMap<&str, String> {
        let mut args = HashMap::new();
        args.insert("token", self.token.clone());
        args.insert("lanIp", self.lan_ip.clone());
        args.insert("macAddress", self.mac_address.clone());
        args.insert("gitHash", self.git_hash.clone());
        args
    }
    pub fn get(&self, pth: &str) -> Result<JsonValue, BoxError> {
        let response = Client::new()
            .get(format!("{}{}", self.url_base, pth))
            .send()?;
        if response.status().is_server_error() {
            // Can't parse json on a server error
            return Ok(JsonValue::new_object());
        }
        Ok(json::parse(response.text()?.as_str())?)
    }
    pub fn put(&self, pth: &str, args: &HashMap<&str, String>) -> Result<JsonValue, BoxError> {
        let client = Client::new();
        let request_url = format!("{}{}", self.url_base, pth);
        let response = client.put(request_url).json(args).send()?;
        if response.status().is_server_error() {
            // Can't parse json on a server error
            return Ok(JsonValue::new_object());
        }
        Ok(json::parse(response.text()?.as_str())?)
    }
    pub fn post(&self, pth: &str, args: &HashMap<&str, String>) -> Result<JsonValue, BoxError> {
        let client = Client::new();
        let request_url = format!("{}{}", self.url_base, pth);
        let response = client.post(request_url).json(args).send()?;
        if response.status().is_server_error() {
            // Can't parse json on a server error
            return Ok(JsonValue::new_object());
        }
        Ok(json::parse(response.text()?.as_str())?)
    }
    // pub fn check_response(&self, response: &Response) -> Result<JsonValue, BoxError> {
    // let status = response.status();
    // let data = JsonValue::new_object();
    // if status.is_server_error() {
    //     // Server error, just return empty json
    //     let dat = JsonValue::new_object();
    //     return Ok(dat);
    // }
    // Ok(json::parse(response.text()?.as_str())?)
    // }
    pub fn get_status(&self) -> Result<JsonValue, BoxError> {
        Ok(self.get("status")?)
    }
    pub fn broadcast_unit_ping(&self) -> Result<JsonValue, BoxError> {
        let args = self.build_def_args();
        Ok(self.put("broadcastUnit/ping", &args)?)
    }
    pub fn activate_room(&self, port: u32) -> Result<JsonValue, BoxError> {
        let mut args = self.build_def_args();
        args.insert("port", format!("{}", port));
        Ok(self.post("room", &args)?)
    }
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
}

#[cfg(test)]

mod test_api {
    use super::*;

    fn build_new_api() -> JamNationApi {
        JamNationApi::new(
            "http://localhost/api/1/",
            "10.10.10.10",
            "test:mac",
            "gitHashString",
        )
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
}
