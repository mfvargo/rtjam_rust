use std::{fs::File, io::Write};

use serde_json::Value;

use super::{box_error::BoxError, jam_packet::JamMessage};

pub struct PacketWriter {
    file: File,
    pub is_writing: bool,
}

impl PacketWriter {
    pub fn new(filename: &str) -> Result<PacketWriter, BoxError> {
        Ok(PacketWriter {
            file: File::create(filename)?,
            is_writing: false,
        })
    }
    pub fn write_message(&mut self, msg: &JamMessage) -> Result<(), BoxError> {
        if self.is_writing {
            self.file.write_all(msg.get_send_buffer())?;
        }
        Ok(())
    }
    pub fn get_status(&self) -> Value {
        let mut state = "idle";
        if self.is_writing {
            state = "recording";
        }
        serde_json::json!({
            "state": state,
            "current_file": ""
        })
    }
}
