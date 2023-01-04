use std::collections::HashMap;

use crate::box_error::BoxError;
use json::JsonValue;
use reqwest::{blocking::Client, blocking::Response, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerStatus {
    id: String,
    name: String,
    description: String,
}

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
    pub fn get(&self, pth: &str) -> Result<Response, BoxError> {
        Ok(Client::new()
            .get(format!("{}{}", self.url_base, pth))
            .send()?)
    }
    pub fn put(&self, pth: &str, args: &HashMap<&str, String>) -> Result<Response, BoxError> {
        let client = Client::new();
        let request_url = format!("{}{}", self.url_base, pth);
        let response = client.put(request_url).json(args).send()?;
        Ok(response)
    }
    pub fn post(&self, pth: &str, args: &HashMap<&str, String>) -> Result<Response, BoxError> {
        let client = Client::new();
        let request_url = format!("{}{}", self.url_base, pth);
        let response = client.post(request_url).json(args).send()?;
        Ok(response)
    }
    pub fn get_status(&self) -> Result<ServerStatus, BoxError> {
        Ok(self.get("status")?.json()?)
    }
    pub fn broadcast_unit_ping(&self) -> Result<JsonValue, BoxError> {
        let args = self.build_def_args();
        Ok(json::parse(
            self.put("broadcastUnit/ping", &args)?.text()?.as_str(),
        )?)
    }
    pub fn activate_room(&self, port: u32) -> Result<JsonValue, BoxError> {
        let mut args = self.build_def_args();
        let pstr = format!("{}", port);
        args.insert("port", pstr);
        let client = Client::new();
        let request_url = format!("{}{}", self.url_base, "room");
        let response = client.post(request_url).json(&args).send()?;
        Ok(json::parse(response.text()?.as_str())?)
    }
    pub fn broadcast_unit_register(&mut self) -> Result<JsonValue, BoxError> {
        let args = self.build_def_args();
        let response = self.post("broadcastUnit", &args)?;
        let status = response.status();
        let result = json::parse(response.text()?.as_str())?;
        if status == StatusCode::OK {
            match result["broadcastUnit"]["token"].as_str() {
                Some(v) => {
                    self.token = String::from(v);
                }
                None => {}
            }
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
    fn get() {
        // you should be able to http get a resource
        let api = build_new_api();
        let result = api.get("status").unwrap();
        assert!(result.status().is_success());
    }
    #[test]
    fn get_status() {
        // You should get the status from the server
        let api = build_new_api();
        let status = api.get_status().unwrap();
        assert_eq!(status.name, "rtjam-nation");
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
