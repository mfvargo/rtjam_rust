use std::{fs::File, io::{Read, Seek, SeekFrom, Write}};

use chrono::{DateTime, Local};
use serde_json::Value;

use super::{box_error::BoxError, jam_packet::{JamMessage, JAM_HEADER_SIZE}};

const MAX_FILE_SIZE: usize = 1 * 1024 * 1024 * 1024;  // 1 Gig max file size
const CHUNK_SIZE: usize = JAM_HEADER_SIZE + 512;   // Size of each network chunk

pub struct PacketWriter {
    file: File,
    filename: String,
    pub is_writing: bool,
    file_size: usize,
}

impl PacketWriter {
    pub fn new(filename: &str) -> Result<PacketWriter, BoxError> {
        Ok(PacketWriter {
            filename: filename.to_string(),
            file: File::create(filename)?,
            is_writing: false,
            file_size: 0,
        })
    }
    pub fn write_message(&mut self, msg: &JamMessage) -> Result<(), BoxError> {
        if self.is_writing && self.file_size < MAX_FILE_SIZE {
            self.file_size += msg.get_send_buffer().len();
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
        let t: DateTime<Local> = metadata.modified().unwrap().into();
        let s: String = t.to_rfc2822();
        serde_json::json!({
            "state": state,
            "file": {
                "name": self.filename,
                "date": s,
                "size": metadata.len(),
                "capacity": metadata.len() as f64 / MAX_FILE_SIZE as f64 * 100.0,
            }
        })
    }
}

pub struct PacketReader {
    file: File,
    filename: String,
    file_chunks: usize,
    offset: usize,
    packet: JamMessage,
    pub now_offset: u128,
}

impl PacketReader {
    // Open a packet dump file and read in the first packet
    pub fn new(filename: &str, now: u128) -> Result<PacketReader, BoxError> {
        let file = File::open(filename)?;
        let metadata = file.metadata()?;
        let size = metadata.len() as usize / CHUNK_SIZE;
        let mut reader = PacketReader {
            file: file,
            filename: filename.to_string(),
            file_chunks: size,
            offset: 0,
            packet: JamMessage::new(),
            now_offset: 0,
        };
        reader.read_packet()?;
        reader.now_offset = now - reader.packet.get_server_time() as u128;
        Ok(reader)
    }
    pub fn get_position(&self) -> usize {
        (self.offset * 100 / (CHUNK_SIZE * self.file_chunks)).clamp(0, 100)
    }
    pub fn get_status(&self) -> Value {
        let metadata = self.file.metadata().unwrap();
        let t: DateTime<Local> = metadata.modified().unwrap().into();
        let s: String = t.to_rfc2822();
        serde_json::json!({
            "state": "playing",
            "offset": self.get_position(),
            "file": {
                "name": self.filename,
                "date": s,
                "size": metadata.len(),
                "capacity": metadata.len() as f64 / MAX_FILE_SIZE as f64 * 100.0,
            }
        })
    }
    pub fn read_packet(&mut self) -> Result<(), BoxError> {
        // read the header
        self.file.read_exact(self.packet.get_header())?;
        let size = self.packet.get_num_audio_chunks() as usize * 32;
        self.file.read_exact(self.packet.get_audio_space(size))?;
        self.packet.set_nbytes(JAM_HEADER_SIZE + size)?;
        self.offset += self.packet.get_nbytes();
        Ok(())
    }

    pub fn seek_to(&mut self, now: u128, loc: usize) -> Result<(), BoxError> {
        self.file.seek(SeekFrom::Start((self.file_chunks * loc / 100 * CHUNK_SIZE) as u64))?;
        self.read_packet()?;
        self.now_offset = now - self.packet.get_server_time() as u128;
        Ok(())
    }

    pub fn get_packet(&self) -> &JamMessage {
        &self.packet
    }

    pub fn read_up_to(&mut self, now: u128) -> Result<Option<JamMessage>, BoxError> {
        if self.micros_till_packet(now) > 0 {
            // The current packet is in the future, nothing to do
            return Ok(None)
        }
        // So the current packet is in the past, clone it for return
        let rval = self.packet.clone();
        // now advance to the next packet
        self.read_packet()?;
        // give them the clone
        Ok(Some(rval))
    }

    pub fn micros_till_packet(&self, now: u128) -> u128 {
        let ptime: u128 = self.now_offset + self.packet.get_server_time() as u128;
        if now > ptime {
            return 0;
        } else {
            return ptime - now;
        }
    }
    
}

#[cfg(test)]
mod stream_test {

    use crate::common::get_micro_time;

    use super::*;
    fn make_a_packet(now: u128) -> JamMessage {
        let mut packet = JamMessage::new();
        packet.set_client_id(40001);  // TODO:  This is some hack for room playback
        packet.set_sequence_num(1); // TODO:  Need this to be monotonically increasing
        packet.set_server_time(now as u64);
        // Need to have the mixer and etc.
        let left: [f32; 128] = [0.0; 128];
        let right: [f32; 128] = [0.0; 128];
        packet.encode_audio(&left, &right);
        packet
    }
    /// Create a test file simulating two parties
    fn make_a_test_file(file_name: &str, now: u128) {
        let mut writer = PacketWriter::new(file_name).unwrap();
        writer.is_writing = true;
        let mut party1 = make_a_packet(now);
        party1.set_client_id(40033);
        party1.set_sequence_num(333);
        let mut party2 = make_a_packet(now + 1000);
        party2.set_client_id(40044);
        party2.set_sequence_num(444);
        for _i in 0..1000 {
            party1.set_server_time(party1.get_server_time() + 2666);
            party1.set_sequence_num(party1.get_sequence_num() + 1);
            writer.write_message(&party1).unwrap();
            party2.set_server_time(party2.get_server_time() + 2666);
            party2.set_sequence_num(party2.get_sequence_num() + 1);
            writer.write_message(&party2).unwrap();
        }
    }

    fn print_packet(packet: &Option<JamMessage>) {
        match packet {
            Some(p) => {
                println!("packet: {}", p);
            }
            None => {
                println!("empty packet");
            }
        }
    }

    #[test]
    fn write_and_then_read_file() {
        let now = get_micro_time();
        let mut writer = PacketWriter::new("tmp/foo.dmp").unwrap();
        writer.is_writing = true;
        let packet = make_a_packet(now);
        writer.write_message(&packet).unwrap();
        // writer.write_message(&packet).unwrap();
        let reader = PacketReader::new("tmp/foo.dmp", get_micro_time()).unwrap();
        let p = reader.get_packet();
        println!("read packet: {}", p);
        println!("now_offset: {}", reader.now_offset);
    }

    #[test]
    fn read_stream_by_time() {
        let now = get_micro_time();
        // Make a file that was recorded 10 seconds ago...
        make_a_test_file("tmp/test_audio.dmp", now - 10_000_000);
        // we have a file now with 2000 packets in it from two players now spaced out at 2666 microseconds
        let mut reader = PacketReader::new("tmp/test_audio.dmp", now).unwrap();
        println!("reader has offset: {}", reader.now_offset);
        // If we read it now, we should not get any data
        let mut packet = reader.read_up_to(now - 1).unwrap();
        print_packet(&packet);
        assert_eq!(reader.micros_till_packet(now-1), 1);
        assert!(packet.is_none());
        // if we move ahead 2667 microseconds, it should give us a packet
        packet = reader.read_up_to(now + 1667).unwrap();
        print_packet(&packet);
        assert!(packet.is_some());
        // Another read should give us the second channel
        packet = reader.read_up_to(now + 1667).unwrap();
        print_packet(&packet);
        assert!(packet.is_some());
        // If we read again, it should be none (since we consumed both channels above)
        packet = reader.read_up_to(now + 1667).unwrap();
        print_packet(&packet);
        assert!(packet.is_none());
    }

    #[test]
    fn seek_on_reader() {
        let now = get_micro_time();
        // Make a file that was recorded 100 seconds ago...
        make_a_test_file("tmp/test_audio.dmp", now - 100_000_000);
        // we have a file now with 2000 packets in it from two players now spaced out at 2666 microseconds
        let mut reader = PacketReader::new("tmp/test_audio.dmp", now).unwrap();
        // Lets go 50% through the file
        reader.seek_to(now, 50).unwrap();
        let mut packet = reader.read_up_to(now - 1).unwrap();
        // Even though we are at 50%, there is no packet for now - 1
        assert!(packet.is_none());
        packet = reader.read_up_to(now + 1667).unwrap();
        print_packet(&packet);
        assert!(packet.is_some());
    }
}