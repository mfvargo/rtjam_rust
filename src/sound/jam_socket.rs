use simple_error::bail;

use crate::common::box_error::BoxError;
use crate::common::jam_packet::JamMessage;
use crate::server::player_list::get_micro_time;
use std::fmt;
use std::net::UdpSocket;

pub struct JamSocket {
    sock: UdpSocket,
    client_id: Option<i64>,
    server: String,
    seq_no: u32,
}

impl JamSocket {
    pub fn build(port: i64) -> Result<JamSocket, BoxError> {
        let sock = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
        sock.set_nonblocking(true)?;
        // make it non-blocking
        Ok(JamSocket {
            sock: sock,
            client_id: None,
            server: String::new(),
            seq_no: 0,
        })
    }
    pub fn connect(&mut self, host: &str, port: i64, id: i64) -> Result<(), BoxError> {
        self.server = format!("{}:{}", host, port);
        self.client_id = Some(id);
        Ok(())
    }
    pub fn disconnect(&mut self) -> () {
        self.server.clear();
        self.client_id = None;
        self.seq_no = 0;
    }
    pub fn is_connected(&self) -> bool {
        !self.client_id.is_none()
    }
    pub fn send(&mut self, packet: &mut JamMessage) -> Result<usize, BoxError> {
        match self.client_id {
            Some(id) => {
                packet.set_sample_rate(0);
                packet.set_num_sub_channels(2);
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
        let sock = JamSocket::build(19990).unwrap();
        println!("sock: {}", sock);
        assert!(true);
    }
    #[test]
    fn connecting() {
        // It should be able to connect
        let mut sock = JamSocket::build(19991).unwrap();
        assert!(!sock.is_connected());
        sock.connect("10.0.0.9", 48481, 3949384).unwrap();
        assert!(sock.is_connected());
        sock.disconnect();
        assert!(!sock.is_connected());
    }
    #[test]
    fn sending() {
        // It shoudl be able to send a jam packet
        let mut sock = JamSocket::build(9993).unwrap();
        let mut packet = JamMessage::build();
        sock.connect("10.0.0.9", 48481, 3949384).unwrap();
        assert_eq!(sock.send(&mut packet).unwrap(), JAM_HEADER_SIZE);
    }
}
