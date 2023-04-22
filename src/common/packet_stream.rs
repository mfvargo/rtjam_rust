use std::{fs::{File, DirEntry}, io::Write};

use chrono::{DateTime, Utc};
use serde_json::Value;

use super::{box_error::BoxError, jam_packet::JamMessage};

pub struct PacketWriter {
    file: File,
    filename: String,
    pub is_writing: bool,
}

impl PacketWriter {
    pub fn new(filename: &str) -> Result<PacketWriter, BoxError> {
        Ok(PacketWriter {
            filename: filename.to_string(),
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
        let metadata = self.file.metadata().unwrap();
        let t: DateTime<Utc> = metadata.modified().unwrap().into();
        let s: String = t.to_rfc2822();
        serde_json::json!({
            "state": state,
            "current_file": {
                "name": self.filename,
                "date": s,
                "size": metadata.len(),
            }
        })
    }
}
