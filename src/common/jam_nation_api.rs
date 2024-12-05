//! REST API to the rtjam-nation to register and activate sound and broadcast elements
//!
//! Super simple.  elements will obtain a token by using the register function for them.
//! This token is then returned so that elements can create websocket chat room with the
//! same name.  This then allows the "meet me in the middle" protocol used by the U/X
use std::collections::HashMap;

use crate::common::box_error::BoxError;
use reqwest::blocking::Client;
use serde_json::{from_str, Value};

use log::{debug, info, warn, /* error */};
// use serde::{Deserialize, Serialize};

/// Define the trait for JamNationApi
pub trait JamNationApiTrait {
    fn get_token(&self) -> &str;
    fn has_token(&self) -> bool;   // Add other methods as needed...
    fn forget_token(&mut self) -> ();
    // TODO: Collapse registration for unit types into single fn
    fn jam_unit_register(&mut self) -> Result<(), BoxError>;
    // TODO: Collapse unit_ping into single call and use self to identify type
    fn broadcast_unit_ping(&self) -> Result<Value, BoxError>;
    fn jam_unit_ping(&self) -> Result<Value, BoxError>;
}

/// The structure that holds state about the api connection
pub struct JamNationApi {
    url_base: String,
    token: String,
    mac_address: String,
    git_hash: String,
    // pub args: NationArgs,
}

impl JamNationApiTrait for JamNationApi {
    /// returns the token for the component once it has registered
    fn get_token(&self) -> &str {
        self.token.as_str()
    }

    /// Indicates the component has successfully registered
    fn has_token(&self) -> bool {
        !self.token.is_empty()
    }

    /// Clear the token. Used if the network fails and reconnects
    fn forget_token(&mut self) -> () {
        self.token = "".to_string();
    }
    
    /// Tell the rtjam-nation the sound component exists
    fn jam_unit_register(&mut self) -> Result<(), BoxError> {
        let mut args = self.build_def_args();
        args.insert("canTalkOnWebsocket", String::from("true"));
        let result = self.post("jamUnit", &args)?;
        if let Some(token) = result["jamUnit"]["token"].as_str() {
            self.token = String::from(token);
        }
        info!("Jam unit registered with token: {}", self.token);
        Ok(())
    }
    
    /// Let the nation know the broadcast component is working
    fn broadcast_unit_ping(&self) -> Result<Value, BoxError> {
        let args = self.build_def_args();
        Ok(self.put("broadcastUnit/ping", &args)?)
    }

    /// Tell the nation the sound component is alive
    fn jam_unit_ping(&self) -> Result<Value, BoxError> {
        let args = self.build_def_args();
        Ok(self.put("jamUnit/ping", &args)?)
    }
}

impl JamNationApi {
    /// Used to manage the connection to the JamNation API
    ///
    /// - url_base: url base for the api.  (something like <http://rtjam-nation.com/api/1/>)
    /// - mac_address: the mac address of the component. Used to uniquely identify the componet
    /// - git_hash: the current git hash string for the build.  Lets the nation know what software component has
    pub fn new(url_base: &String, mac_address: &String, git_hash: &String) -> JamNationApi {
        debug!("JamNationApi::new called url_base: {}, mac_address: {}, git_hash: {}", url_base, mac_address, git_hash);
        JamNationApi {
            token: String::new(),
            url_base: url_base.to_string(),
            mac_address: mac_address.to_string(),
            git_hash: git_hash.to_string(),
        }
    }

    /// Tell the rtjam-nation the broadcast component exists
    pub fn broadcast_unit_register(&mut self) -> Result<Value, BoxError> {
        let args = self.build_def_args();
        let result = self.post("broadcastUnit", &args)?;
        match result["broadcastUnit"]["token"].as_str() {
            Some(v) => {
                self.token = String::from(v);
            }
            None => {}
        }
        info!("Broadcast unit registered with token: {}", self.token);
        Ok(result)
    }

    /// Tell the rtjam-nation that the broadcast component has a room
    pub fn activate_room(&self, port: u32) -> Result<Value, BoxError> {
        let mut args = self.build_def_args();
        args.insert("port", format!("{}", port));
        Ok(self.post("room", &args)?)
    }

    fn post(&self, pth: &str, args: &HashMap<&str, String>) -> Result<Value, BoxError> {
        let client = Client::new();
        let request_url = format!("{}{}", self.url_base, pth);
        let response = client.post(request_url).json(args).send()?;
        if response.status().is_server_error() {
            // Can't parse json on a server error
            warn!("jam_nation_api::post server error: {}", response.text().unwrap_or("Unknown error".to_string()));
            return Err(BoxError::from("Server error occurred"));
        }
        Ok(from_str(response.text()?.as_str())?)
    }

    fn put(&self, pth: &str, args: &HashMap<&str, String>) -> Result<Value, BoxError> {
        let client = Client::new();
        let request_url = format!("{}{}", self.url_base, pth);
        let response = client.put(request_url).json(args).send()?;
        if response.status().is_server_error() {
            // Can't parse json on a server error
            warn!("jam_nation_api::put server error: {}", response.text().unwrap_or("Unknown error".to_string()));
            return Err(BoxError::from("Server error occurred"));
        }
        Ok(from_str(response.text()?.as_str())?)
    }
    
    fn build_def_args(&self) -> HashMap<&str, String> {
        let mut args = HashMap::new();
        args.insert("token", self.token.clone());
        args.insert("lanIp", "10.0.0.0".to_string());
        args.insert("macAddress", self.mac_address.clone());
        args.insert("gitHash", self.git_hash.clone());
        args
    }

    // fn get(&self, pth: &str) -> Result<JsonValue, BoxError> {
    //     let response = Client::new()
    //         .get(format!("{}{}", self.url_base, pth))
    //         .send()?;
    //     if response.status().is_server_error() {
    //         // Can't parse json on a server error
    //         return Ok(JsonValue::new_object());
    //     }
    //     Ok(json::parse(response.text()?.as_str())?)
    // }


    // /// call this to just see if the API endpoint is working.
    // pub fn get_status(&self) -> Result<JsonValue, BoxError> {
    //     Ok(self.get("status")?)
    // }

}

#[cfg(test)]
mod test_api {
    use super::*;

    fn build_new_api() -> JamNationApi {
        JamNationApi::new(
            &String::from("http://localhost:8080/api/1/"), 
            &String::from("test:mac"), 
            &String::from("gitHashString")
        )
    }

    #[test]
    fn test_broadcast_unit_register_token() {
        let mut api = build_new_api();
        let result = api.broadcast_unit_register();  
        assert!(result.is_ok(), "Test not expecting an error on broadcast_unit_register()");
        // If the result is Ok, convert to a Map for further checking of content
        let result_unwrapped = result.unwrap();
        let result_map = result_unwrapped.as_object().expect("Result should be an object");
        let broadcast_unit = result_map.get("broadcastUnit").expect("Result should have a 'broadcastUnit' field");
        let token = broadcast_unit.get("token").expect("Result['broadcastUnit'] should have a 'token' field");
        // Assert the token value
        assert_eq!(token.as_str().unwrap(), api.get_token(), "Token should be returned");
    }

    #[test]
    fn test_activate_room() {
        let api = build_new_api();    
        let result = api.activate_room(7891);  
        assert!(result.is_ok(), "Test not expecting an error on activate_room()");
        // If the result is Ok, unwrap it safely and assert the value
        // If the result is Ok, convert to a Map for further checking of content
        let result_unwrapped = result.unwrap();
        let result_map = result_unwrapped.as_object().expect("Result should be an object");
        let status = result_map.get("status").expect("Result should have a 'status' field");
        assert_eq!(status.as_str().unwrap(), "activated", "result['status'] should be 'activated'");
    }

    // TODO: A whole lot more UTs
    // #[test]
    // fn get_status() {
    //     // You should get the status from the server
    //     let api = build_new_api();
    //     let status = api.get_status().unwrap();
    //     assert_eq!(status["name"], "rtjam-nation");
    // }

    // #[test]
    // fn test_broadcast_unit_ping_id() {
    //     let mut api = build_new_api();
    //     let ping = api.broadcast_unit_ping().unwrap();
    //     assert!(!ping["broadcastUnit"]["id"].as_str().is_empty());
    // }

    // #[test]
    // fn jam_unit_register_and_ping() {
    //     let mut api = build_new_api();
    //     let reg = api.jam_unit_register().unwrap();
    //     assert_eq!(reg["jamUnit"]["token"], api.get_token());
    //     let ping = api.jam_unit_ping().unwrap();
    //     assert!(!ping["jamUnit"]["id"].is_empty());
    // }
}
