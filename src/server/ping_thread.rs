use log::{error, warn, debug};

// Weird thing.  YOu ahve to have the import of JamNationApiTrait for it to compile, but
// always generate a warning about it being unused.  Not sure what to do about this.
#[allow(unused_imports)]
use crate::common::jam_nation_api::JamNationApiTrait;
use crate::common::{
    jam_nation_api::JamNationApi,
    box_error::BoxError,
};
use std::{
    thread::sleep,
    time::Duration,
};

pub fn broadcast_ping_thread(mut api: JamNationApi, port: u32) -> Result<(), BoxError> {
    loop {
        while api.has_token() == true {
            // While in this loop, we are going to ping every 10 seconds
            match api.broadcast_unit_ping() {
                Ok(ping) => {
                    if ping["broadcastUnit"].is_null() {
                        // Error in the ping.  better re-register
                        warn!("Ping error");
                        api.forget_token();
                    } else {
                        // Successful ping.. Sleep for 10
                        debug!("ping success");
                        sleep(Duration::new(10, 0));
                    }
                }
                Err(e) => {
                    api.forget_token();
                    error!("api error: {}", e);
                }
            }
        }
        if !api.has_token() {
            // We need to register the server
            match api.broadcast_unit_register() {
                Ok(_res) => {
                    // Activate the room
                    let _room_activate = api.activate_room(port);
                }
                Err(e) => {
                    warn!("cannot register with server: {}", e);
                }
            }
        }
        // This is the timer between registration attempts
        sleep(Duration::new(2, 0));
    }
}
