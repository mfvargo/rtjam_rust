//! module to organized the recording of room contents to files
use super::box_error::BoxError;
use chrono::{DateTime, Local};
use log::info;
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
    pub fn has_file(&self, name: &str) -> bool {
        if let Some(_position) = self.recordings.iter().position(|x| x.file_name == name) {
            true
        } else {
            false
        }
    }
    pub fn add_file(&mut self, from: &str, to: &str) -> () {
        let mut to_file: String = to.to_string();
        let mut count = 1;
        while self.has_file(&to_file) {
            to_file = format!("{}_{}", to, count);
            count += 1;
        }
        let res = std::fs::copy(
            from,
            format!("{}/{}", self.path, to_file.as_str())
        );
        info!("copy result: {:?}", res);
        self.dirty = true;
    }
    pub fn delete_file(&mut self, filename: &str) -> () {
        let _res = std::fs::remove_file(format!("{}/{}", self.path, filename));
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

    use std::fs::OpenOptions;

    use super::*;

 
    fn touch(path: &str) -> std::io::Result<()> {
        let _file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)?;
        Ok(())
    }
    fn build_catalog() -> RecordingCatalog {
        let _res = std::fs::remove_dir_all("test_catalog");
        std::fs::create_dir("test_catalog").unwrap();
        touch("test_catalog/test1.raw").unwrap();
        touch("test_catalog/test2.raw").unwrap();
        RecordingCatalog::new("test_catalog").unwrap()
    }
    #[test]
    fn should_find() {
        let cat = build_catalog();
        let rec = cat.recordings.get(0).unwrap();
        assert!(cat.has_file(rec.file_name.as_str()));
        assert!(!cat.has_file("bogus_name_not_there.raw"));
    }
    #[test]
    fn should_add() {
        let mut cat = build_catalog();
        let rec = cat.recordings.get(0).unwrap();
        let count = cat.len();
        let dup_file = rec.file_name.clone();
        cat.add_file(format!("{}/{}", "test_catalog", dup_file).as_str(), &dup_file);
        cat.load_recordings().unwrap();
        assert_eq!(cat.len(), count + 1);
        assert!(cat.has_file(dup_file.as_str()));
    }
    #[test]
    fn should_json() {
        let mut cat = build_catalog();
        println!("as_json: {}", cat.as_json());
    }
}
