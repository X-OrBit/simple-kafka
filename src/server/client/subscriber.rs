use log::debug;
use std::io;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast::Receiver;
use tokio::time::timeout;

/// A `Subscriber` is one of the `Client` types being served
///
/// `Subscriber` keeps it in itself `socket_addr` (ip/port of connected `Client`) and `topic` (connected `Topic` name)
///
/// The `Subscriber` client listens to messages sent by each [Publisher](crate::server::client::publisher::Publisher)
/// that are connected to the same topic as the `Subscriber`
pub struct Subscriber {
    pub socket_addr: SocketAddr,
    pub topic: String,
    receiver: Receiver<Vec<u8>>,
}

impl Subscriber {
    pub fn new(socket_addr: SocketAddr, topic: String, receiver: Receiver<Vec<u8>>) -> Self {
        Self {
            socket_addr,
            topic,
            receiver,
        }
    }

    /// Start listening messages from [Publishers](crate::server::client::publisher::Publisher)
    pub async fn listen(&mut self, stream: &mut TcpStream) -> io::Result<()> {
        loop {
            let data = timeout(Duration::from_secs_f64(1.0), self.receiver.recv()).await;
            let data = match data {
                Ok(data) => data,
                Err(_) => {
                    let result = timeout(Duration::from_secs_f64(0.2), async {
                        let mut buf = [0u8; 1];
                        let peak_result = stream.peek(&mut buf).await;
                        peak_result.is_ok() && peak_result.unwrap() > 0usize
                    })
                    .await;
                    match result {
                        Ok(result) => {
                            if result {
                                continue;
                            }
                            debug!("Disconnect {}", &self.socket_addr);
                            break Ok(()); // disconnected
                        }
                        Err(_) => continue,
                    };
                }
            };

            match data {
                Err(_) => break Err(Error::from(ErrorKind::BrokenPipe)),
                Ok(message) => match stream.write(message.as_slice()).await {
                    Ok(_) => {
                        debug!(
                            "Send message to subscriber {}: \"{}\"",
                            self.socket_addr,
                            String::from_utf8_lossy(message.as_slice())
                                .strip_suffix("\n")
                                .unwrap()
                        );
                    }
                    Err(e) if e.kind() == ErrorKind::ConnectionAborted => {
                        debug!("Aborted disconnect {}", &self.socket_addr);
                        break Ok(());
                    }
                    Err(e) => break Err(Error::from(e)),
                },
            }
        }
    }
}
