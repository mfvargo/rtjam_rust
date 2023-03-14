//! chunk of bytes that has a slice of audio
//!
//! This is the stuff that gets put "on the wire" between broadcast and sound components
//! It is very intentionally simple.  The idea is NOT to have lots of different sample
//! rates and encoding schemes.  For the real time audio to work the networks have
//! to be fast.  There is nothing to be gain by compression or supporting variable rates.
use byteorder::{ByteOrder, NetworkEndian};
use simple_error::bail;
use std::fmt;

use super::box_error::BoxError;

pub const JAM_BUF_SIZE: usize = 1024;
/// the message that gets read/write on the udp socket
///
/// super simple by design.  just has getters/setters to make sure everything
/// is network endian and packed tight.
pub struct JamMessage {
    buffer: [u8; JAM_BUF_SIZE],
    nbytes: usize,
}

// The original C Definition for doc purposes
// struct JamMessage
// {
//   uint8_t Channel;         // Assigned by server
//   uint8_t SampleRate;      // Assigned by client
//   uint8_t NumSubChannels;  // Assigned by client
//   uint8_t Beat;            // Assigned by server for shared synchonized metronome
//   uint64_t ServerTime;     // Assigned by server
//   uint64_t TimeStamp;      // Assigned by the client
//   uint32_t ClientId;       // Assigned by the client to know which channel is their channel
//   uint32_t SequenceNumber; // Assigned by client
//   unsigned char buffer[JAM_BUF_SIZE];
// };
// Header size is all the stuff up to the buffer...
pub const JAM_HEADER_SIZE: usize = 1 + 1 + 1 + 1 + 8 + 8 + 4 + 4;

impl JamMessage {
    /// build a message
    pub fn new() -> JamMessage {
        JamMessage {
            buffer: [0; JAM_BUF_SIZE],
            nbytes: JAM_HEADER_SIZE,
        }
    }
    /// Not used at the present.  I think this could be used to host multiple
    /// rooms on the same broadcast server.
    pub fn get_channel(&self) -> u8 {
        self.buffer[0]
    }
    /// Not used see get_channel
    pub fn set_channel(&mut self, chan: u8) -> () {
        self.buffer[0] = chan;
    }
    /// Not used
    pub fn get_sample_rate(&self) -> u8 {
        self.buffer[1]
    }
    /// Not used
    pub fn set_sample_rate(&mut self, r: u8) -> () {
        self.buffer[1] = r;
    }
    /// Not used
    pub fn get_num_sub_channels(&self) -> u8 {
        self.buffer[2]
    }
    /// Not used
    pub fn set_num_sub_channels(&mut self, n: u8) -> () {
        self.buffer[2] = n;
    }
    /// Get the metronome beat from the server
    pub fn get_beat(&self) -> u8 {
        self.buffer[3]
    }
    /// used by broadcast to send a synchornized metronome beat to the room
    pub fn set_beat(&mut self, b: u8) -> () {
        self.buffer[3] = b;
    }
    /// Get the timestamp set by the broacast element (used for round trip delay calcs)
    pub fn get_server_time(&self) -> u64 {
        NetworkEndian::read_u64(&self.buffer[4..12])
    }
    /// Set the broadcast time
    pub fn set_server_time(&mut self, t: u64) -> () {
        NetworkEndian::write_u64(&mut self.buffer[4..12], t)
    }
    /// client timestamp (sound element)
    pub fn get_client_timestamp(&self) -> u64 {
        NetworkEndian::read_u64(&self.buffer[12..20])
    }
    /// client timestamp (sound element)
    pub fn set_client_timestamp(&mut self, t: u64) -> () {
        NetworkEndian::write_u64(&mut self.buffer[12..20], t)
    }
    /// retrieve the id of the client (sound element) who made this packet
    pub fn get_client_id(&self) -> u32 {
        NetworkEndian::read_u32(&self.buffer[20..24]) // 12 - 16 is the offset for the ClientId
    }
    /// set the id on the packet used by sound element
    pub fn set_client_id(&mut self, id: u32) -> () {
        NetworkEndian::write_u32(&mut self.buffer[20..24], id)
    }
    /// Get the client assigned sequence number (used to detect lost and out of seq packets)
    pub fn get_sequence_num(&self) -> u32 {
        NetworkEndian::read_u32(&self.buffer[24..28]) // 12 - 16 is the offset for the ClientId
    }
    /// client (sound element) assigns a sequence number to each packet
    pub fn set_sequence_num(&mut self, id: u32) -> () {
        NetworkEndian::write_u32(&mut self.buffer[24..28], id)
    }
    /// Get the address of the buffer (used for writing from network into buffer)
    pub fn get_buffer(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
    /// Get the slice of buffer that has some data
    pub fn get_send_buffer(&self) -> &[u8] {
        &self.buffer[0..self.nbytes]
    }
    /// Encode two channes of audio into a buffer
    ///
    /// coding is u16 offset  (0 => -1.0 and 65535 => +1.0)
    pub fn encode_audio(&mut self, chan1: &[f32], chan2: &[f32]) -> usize {
        // this will take an array of floats and encode them into the packet

        let mut idx = JAM_HEADER_SIZE;
        for v in chan1 {
            NetworkEndian::write_u16(&mut self.buffer[idx..idx + 2], Self::convert_to_u16(v));
            idx += 2; // move ahead 2 bytes
        }
        for v in chan2 {
            NetworkEndian::write_u16(&mut self.buffer[idx..idx + 2], Self::convert_to_u16(v));
            idx += 2; // move ahead 2 bytes
        }
        self.nbytes = idx;
        idx
    }
    fn convert_to_u16(v: &f32) -> u16 {
        let mut sample = v + 1.0;
        // Prevent clipping
        if sample > 2.0 {
            sample = 2.0;
        }
        if sample < 0.0 {
            sample = 0.0;
        }
        (sample * 32766.0) as u16
    }
    /// decode the audio contained in the message into two f32 vectors
    ///
    /// These vectors will get shoved into jitterbuffers
    pub fn decode_audio(&self) -> (Vec<f32>, Vec<f32>) {
        let mut chan_1: Vec<f32> = Vec::new();
        let mut chan_2: Vec<f32> = Vec::new();
        let num_samples = (self.nbytes - JAM_HEADER_SIZE) / 4; //  2 bytes per sample and 2 channels of data
        let mut off_1 = JAM_HEADER_SIZE; // starting offset to first channel
        let mut off_2 = JAM_HEADER_SIZE + num_samples * 2; // staring offset to 2nd channel
        for _n in 0..num_samples {
            chan_1.push(Self::convert_to_f32(NetworkEndian::read_u16(
                &self.buffer[off_1..off_1 + 2],
            )));
            chan_2.push(Self::convert_to_f32(NetworkEndian::read_u16(
                &self.buffer[off_2..off_2 + 2],
            )));
            off_1 += 2;
            off_2 += 2;
        }
        (chan_1, chan_2)
    }
    fn convert_to_f32(n: u16) -> f32 {
        (1.0 / 32768.0 * n as f32) - 1.0
    }
    /// set the number of bytes on the packet (so when we read one, this says how much we read)
    pub fn set_nbytes(&mut self, amt: usize) -> Result<(), BoxError> {
        if !self.is_valid(amt) {
            bail!("invalid packet");
        }
        self.nbytes = amt;
        Ok(())
    }
    /// some simple sanity checks on the message to make sure it makes sense
    pub fn is_valid(&self, amt: usize) -> bool {
        // a packet has to be at least as big as a header and must be an even number of bytes
        amt >= JAM_HEADER_SIZE && amt % 2 == 0
    }
}

impl fmt::Display for JamMessage {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ seq: {}, client: {}, cl_time: {}, sv_time: {}, nbytes: {} }}",
            self.get_sequence_num(),
            self.get_client_id(),
            self.get_client_timestamp(),
            self.get_server_time(),
            self.nbytes
        )
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn build() {
        // You should be able to build a JamMessage
        let mut msg = JamMessage::new();
        msg.set_channel(33);
        assert_eq!(msg.get_channel(), 33);
    }
    #[test]
    fn beat() {
        // You should get the client id from the packet
        let mut msg = JamMessage::new();
        msg.set_beat(4);
        assert_eq!(msg.get_beat(), 4);
    }
    #[test]
    fn client_id() {
        // You should get the client id from the packet
        let mut msg = JamMessage::new();
        msg.set_client_id(32);
        assert_eq!(msg.get_client_id(), 32);
    }
    #[test]
    fn server_timestamps() {
        // You should get the client id from the packet
        let mut msg = JamMessage::new();
        msg.set_server_time(4432);
        assert_eq!(msg.get_server_time(), 4432);
    }
    #[test]
    fn client_timestamps() {
        // You should get the client id from the packet
        let mut msg = JamMessage::new();
        msg.set_client_timestamp(7737);
        assert_eq!(msg.get_client_timestamp(), 7737);
    }
    #[test]
    fn get_buffer() {
        // You should be able to get a mutable ref to the buffer
        let mut msg = JamMessage::new();
        let buf = msg.get_buffer();
        assert_eq!(buf[0], 0);
    }
    #[test]
    fn is_valid() {
        // it should tell you if it's valid based on the number of bytes
        let msg = JamMessage::new();
        assert_eq!(msg.is_valid(0), false);
        assert_eq!(msg.is_valid(JAM_HEADER_SIZE + 5), false);
        // This is for a packet with 64 samples each 2 bytes wide with 2 channels
        assert_eq!(msg.is_valid(JAM_HEADER_SIZE + 64 * 2 * 2), true)
    }
    #[test]
    fn encode_audio() {
        // It should take two channels and encode them into the JamPacket
        let chan_1: Vec<f32> = vec![0.5; 128];
        let chan_2: Vec<f32> = vec![0.6; 128];
        let mut msg = JamMessage::new();
        assert_eq!(
            msg.encode_audio(&chan_1[..], &chan_2[..]),
            256 * 2 + JAM_HEADER_SIZE
        );
        let (dec_1, dec_2) = msg.decode_audio();
        assert_eq!(dec_1.len(), 128);
        assert_eq!(dec_2.len(), 128);
    }
}
