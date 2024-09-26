//! module to organized the recording of room contents to files
use super::box_error::BoxError;
use chrono::{DateTime, Local};
use serde_json::Value;
use simple_error::bail;
use std::{fs::read_dir, path::Path, time::SystemTime};

/// Represents a file
pub struct Recording {
    file_name: String,
    time: SystemTime,
    size: u64,
}

impl Recording {
    pub fn new(filename: &str, time: SystemTime, size: u64) -> Recording {
        Recording {
            file_name: filename.to_string(),
            time: time,
            size: size,
        }
    }
    pub fn as_json(&self) -> Value {
        let t: DateTime<Local> = self.time.into();
        let s: String = t.to_rfc2822();
        serde_json::json!({
            "name": self.file_name,
            "date": s,
            "size": self.size,
        })
    }
}

pub struct RecordingCatalog {
    path: String,
    recordings: Vec<Recording>,
    dirty: bool,
}

impl RecordingCatalog {
    pub fn new(path: &str) -> Result<RecordingCatalog, BoxError> {
        let file_path = Path::new(path);
        if !file_path.is_dir() {
            bail!(format!("{} path is not a directory!", path));
        }
        let mut cat = RecordingCatalog {
            path: path.to_string(),
            recordings: vec![],
            dirty: true,
        };
        cat.load_recordings()?;
        Ok(cat)
    }
    pub fn load_recordings(&mut self) -> Result<(), BoxError> {
        self.dirty = true;
        self.recordings.clear();
        for entry in read_dir(&self.path)? {
            let entry = entry?;
            // dbg!(&entry.file_type()?);
            let metadata = entry.metadata()?;
            if !metadata.is_dir() {
                self.recordings.push(Recording::new(
                    entry.file_name().to_str().unwrap(),
                    metadata.modified()?,
                    metadata.len(),
                ))
            }
        }
        Ok(())
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub fn add_file(&mut self, filename: &str) -> () {
        let t: DateTime<Local> = SystemTime::now().into();
        let s: String = t.format("%H:%M:%S").to_string();
        let _res = std::fs::copy(
            filename, 
            format!("recs/audio_{}.raw", s));
        self.dirty = true;
    }
    pub fn delete_file(&mut self, filename: &str) -> () {
        let _res = std::fs::remove_file(format!("recs/{}", filename));
        self.dirty = true;
    }
    pub fn len(&self) -> usize {
        self.recordings.len()
    }
    pub fn as_json(&mut self) -> Value {
        self.dirty = false;
        let mut recs: Vec<Value> = vec![];
        for rec in &self.recordings {
            recs.push(rec.as_json());
        }
        serde_json::json!(recs)
    }
}

#[cfg(test)]
mod recording_test {

    use super::*;

    #[test]
    fn should_build() {
        if let Ok(cat) = RecordingCatalog::new("test_catalog") {
            assert_eq!(cat.path, "test_catalog");
            assert_eq!(cat.len(), 2);
        } else {
            assert!(true);
        }
    }
    #[test]
    fn should_json() {
        if let Ok(mut cat) = RecordingCatalog::new("test_catalog") {
            println!("as_json: {}", cat.as_json());
        } else {
            assert!(true);
        }
    }
}
