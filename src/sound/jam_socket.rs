//! UDP socket use to send and receive audio data to the room
//!
//! although the socke it UDP based, it does have a connect/disconnect
//! function which basically holds the state of which server the thing
//! is pointed at.
//!
//! the socket is nonblocking which allows the sound side to poll for packets
//! on the audio process loop, stuff them into buffers, and then continue on.
//! So the network data is buffered in the socket until the process loop comes
//! around and then gets shoved into jitter buffer.  Then the process loop will
//! pull out a mix and feed it to the audio output.
//!
//! This prevents the jitter buffer from having to have any mutexes. (one writer, one reader)
use simple_error::bail;

use crate::common::{box_error::BoxError, get_micro_time, jam_packet::JamMessage, sock_with_tos};
use std::fmt;
use std::net::UdpSocket;

/// re-connectable udp socket to talk to the broadcast server
pub struct JamSocket {
    sock: UdpSocket,
    client_id: Option<i64>,
    server: String,
    seq_no: u32,
}

impl JamSocket {
    /// Build a new socket
    pub fn new(port: i64) -> Result<JamSocket, BoxError> {
        let sock = sock_with_tos::new(port as u32);
        sock.set_nonblocking(true)?;
        // make it non-blocking
        Ok(JamSocket {
            sock: sock,
            client_id: None,
            server: String::new(),
            seq_no: 0,
        })
    }
    /// Connect the socket to a specific broadcast unit
    pub fn connect(&mut self, host: &str, port: i64, id: i64) -> Result<(), BoxError> {
        self.server = format!("{}:{}", host, port);
        self.client_id = Some(id);
        Ok(())
    }
    /// clear out server state data.
    pub fn disconnect(&mut self) -> () {
        self.server.clear();
        self.client_id = None;
        self.seq_no = 0;
    }
    /// Are we currently linked to a broadcast unit
    pub fn is_connected(&self) -> bool {
        !self.client_id.is_none()
    }
    /// Send a JamMesssage to the room.
    pub fn send(&mut self, packet: &mut JamMessage) -> Result<usize, BoxError> {
        match self.client_id {
            Some(id) => {
                packet.set_sample_rate(0);
                packet.set_client_id(id as u32);
                packet.set_sequence_num(self.seq_no);
                self.seq_no += 1;
                packet.set_client_timestamp(get_micro_time() as u64);
                Ok(self
                    .sock
                    .send_to(packet.get_send_buffer(), self.server.as_str())?)
            }
            None => {
                bail!("socket not connected");
            }
        }
    }
    /// Read a packet into a JamMessage,  returns an Err result if there is nothing there to read.
    pub fn recv(&self, packet: &mut JamMessage) -> Result<(), BoxError> {
        let (nbytes, _addr) = self.sock.recv_from(packet.get_buffer())?;
        packet.set_nbytes(nbytes)?;
        Ok(())
    }
}

impl fmt::Display for JamSocket {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ sock: {} }}", self.sock.local_addr().unwrap())
    }
}
#[cfg(test)]
mod test_jam_socket {
    use crate::common::jam_packet::JAM_HEADER_SIZE;

    use super::*;

    #[test]
    fn build_socket() {
        let _sock = JamSocket::new(19990).unwrap();
        assert!(true);
    }
    #[test]
    fn connecting() {
        // It should be able to connect
        let mut sock = JamSocket::new(19991).unwrap();
        assert!(!sock.is_connected());
        sock.connect("10.0.0.9", 48481, 3949384).unwrap();
        assert!(sock.is_connected());
        sock.disconnect();
        assert!(!sock.is_connected());
    }
    #[test]
    fn sending() {
        // It shoudl be able to send a jam packet
        let mut sock = JamSocket::new(9993).unwrap();
        let mut packet = JamMessage::new();
        sock.connect("10.0.0.9", 48481, 3949384).unwrap();
        assert_eq!(sock.send(&mut packet).unwrap(), JAM_HEADER_SIZE);
    }
}
