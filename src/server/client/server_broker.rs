use crate::server::client::{Client, ClientConnectionError, ClientType, Publisher, Subscriber};
use crate::server::connection_message::ConnectionMessage;
use crate::server::topic::Topic;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

pub const DELIMITER: u8 = 10;

/// ServerBroker is auxiliary Server object
///
/// In a nutshell it allows you to conveniently create a client when connected via:
/// - creating topics
/// - creating communication channels between publishers and subscribers
/// ```
pub(crate) struct ServerBroker {
    topics: Arc<Mutex<HashMap<String, Topic>>>,
}

impl ServerBroker {
    pub fn default() -> Self {
        Self {
            topics: Arc::new(Mutex::new(HashMap::default())),
        }
    }

    async fn get_topic(&self, topic: &str) -> Topic {
        let mut topics_lock = self.topics.lock().await;
        if !topics_lock.contains_key(topic) {
            topics_lock.insert(String::from(topic), Topic::new(String::from(topic)).await);
        }
        topics_lock.get_mut(topic).unwrap().clone()
    }

    async fn get_receiver(&self, topic: &str) -> Receiver<Vec<u8>> {
        self.get_topic(topic).await.sender.subscribe()
    }

    async fn get_sender(&self, topic: &str) -> Sender<Vec<u8>> {
        self.get_topic(topic).await.sender.clone()
    }

    /// Creates [Client] according to the first message received from the connection
    ///
    /// # Errors
    /// - [ClientConnectionError::UnexpectedMessage] if connection message is not in correct format
    /// - [ClientConnectionError::Aborted] if connection is aborted
    /// - [ClientConnectionError::ReaderError] if some other [TcpStream::read] error occurs
    pub async fn new_client(
        &self,
        socket_addr: SocketAddr,
        buf_reader: &mut BufReader<&mut TcpStream>,
    ) -> Result<Client, ClientConnectionError> {
        let mut buffer: Vec<u8> = vec![];

        match buf_reader.read_until(DELIMITER, &mut buffer).await {
            Ok(0) => Err(ClientConnectionError::Aborted),
            Ok(sz) => {
                if buffer[sz - 1] != DELIMITER {
                    return Err(ClientConnectionError::UnexpectedMessage(buffer));
                }
                buffer.pop();
                let message = String::from_utf8_lossy(&buffer);
                match serde_json::from_str::<ConnectionMessage>(&message) {
                    Ok(connection_message) => match connection_message.client_type {
                        ClientType::Subscriber => Ok(Client::Subscriber(Subscriber::new(
                            socket_addr,
                            connection_message.topic.clone(),
                            self.get_receiver(&connection_message.topic).await,
                        ))),
                        ClientType::Publisher => Ok(Client::Publisher(Publisher::new(
                            socket_addr,
                            connection_message.topic.clone(),
                            self.get_sender(&connection_message.topic).await,
                        ))),
                    },
                    Err(_) => Err(ClientConnectionError::UnexpectedMessage(buffer.to_vec())),
                }
            }
            Err(e) if e.kind() == ErrorKind::ConnectionAborted => {
                Err(ClientConnectionError::Aborted)
            }
            Err(_) => Err(ClientConnectionError::ReaderError),
        }
    }
}
