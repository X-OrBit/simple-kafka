use log::debug;
use std::io;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::broadcast::Sender;

/// A `Publisher` is one of the `Client` types being served
///
/// `Publisher` keeps it in itself `socket_addr` (ip/port of connected `Client`) and `topic` (connected `Topic` name)
///
/// The `Publisher` client listens to messages sent by client and sends them to
/// [Subscribers](crate::server::client::subscriber::Subscriber) that are connected to the same topic as the `Publisher`
pub struct Publisher {
    pub socket_addr: SocketAddr,
    pub topic: String,
    sender: Sender<Vec<u8>>,
}

impl Publisher {
    pub fn new(socket_addr: SocketAddr, topic: String, sender: Sender<Vec<u8>>) -> Self {
        Self {
            socket_addr,
            topic,
            sender,
        }
    }

    /// Start listening messages from `Publisher` client and sending it to connected
    /// [Subscribers](crate::server::client::subscriber::Subscriber)
    ///
    /// # Errors
    /// - If somehow read data is has invalid ending, function will return [ErrorKind::InvalidData]
    /// - If server cannot read data from client (disconnection with error, etc.),
    /// function will return error from [TcpStream::read]
    /// - If publisher-subscriber channel will aborted, function will return [ErrorKind::BrokenPipe]
    pub async fn listen(&mut self, buf_reader: &mut BufReader<&mut TcpStream>) -> io::Result<()> {
        loop {
            let mut buffer: Vec<u8> = vec![];
            let sz = match buf_reader
                .read_until(crate::server::client::server_broker::DELIMITER, &mut buffer)
                .await
            {
                Ok(0) => break Ok(()), // publisher disconnection
                Ok(sz) => sz,
                Err(e) if e.kind() == ErrorKind::ConnectionAborted => break Ok(()), // publisher disconnection
                Err(e) => break Err(Error::from(e)),
            };

            debug!(
                "Received message from publisher {}: \"{}\"",
                self.socket_addr,
                String::from_utf8_lossy(buffer.as_slice())
                    .strip_suffix("\n")
                    .unwrap_or("")
            );

            if sz == 0 {
                break Ok(());
            }
            if buffer[sz - 1] != crate::server::client::server_broker::DELIMITER {
                break Err(Error::from(ErrorKind::InvalidData));
            }

            match self.sender.send(buffer) {
                Ok(_) => {}
                Err(_) => break Err(Error::from(ErrorKind::BrokenPipe)),
            }
        }
    }
}
