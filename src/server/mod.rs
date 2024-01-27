pub mod client;
pub mod connection_message;
pub(crate) mod topic;

extern crate tokio;

use log::{error, info};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::BufReader;

use crate::server::client::server_broker::ServerBroker;
use crate::server::client::{Client, ClientConnectionError};
use tokio::io;
use tokio::net::TcpListener;

/// `Server` is Async simplified software message broker server
///
/// `Server` listening on `socket_address` with an automatically created `listener: TcpListener`
///
/// To start the `Server` use the [Server::run] method
///
/// # Examples
/// ```
/// #[tokio::main]
/// async fn main() -> io::Result<()> {
///     let server = Server::new(SocketAddr::new(IpAddr::new("127.0.0.1"), 7000)).await?;
///     server.run().await?;
/// }
/// ```
pub struct Server {
    socket_address: SocketAddr,
    listener: TcpListener,
}

impl Server {
    /// Create and launch new `Server` on `socket_address`
    ///
    /// # Errors
    /// All errors are inherited from [TcpListener::bind]
    pub async fn new(socket_address: &SocketAddr) -> io::Result<Self> {
        Ok(Self {
            socket_address: socket_address.clone(),
            listener: TcpListener::bind(socket_address).await?,
        })
    }

    /// Returns the port on which the server is running.
    /// It is useful if you specify `port=0` when creating a new [Server]
    pub fn port(&self) -> u16 {
        self.listener.local_addr().unwrap().port()
    }

    /// Run created `Server`
    ///
    /// While the `Server` is running, it listens for new connections of `Publishers`/`Subscribers` and
    /// sending messages from `Publishers` to `Subscribers` inside one topic via [ServerBroker]
    ///
    /// # Errors
    /// All errors are inherited from [TcpListener::accept]
    pub async fn run(&mut self) -> io::Result<()> {
        info!("Start kafka server on address {}", self.socket_address);
        let origin_server_broker = Arc::new(ServerBroker::default());

        loop {
            let server_broker = origin_server_broker.clone();

            let (mut stream, socket_addr) = self.listener.accept().await?;

            tokio::spawn(async move {
                info!("Client {} connected to server", socket_addr);

                let mut buf_reader = BufReader::new(&mut stream);

                let client = match server_broker.new_client(socket_addr, &mut buf_reader).await {
                    Ok(client) => client,
                    Err(e) => {
                        match e {
                            ClientConnectionError::UnexpectedMessage(message) => info!(
                                r#"Failed to parse connection message from client {}: "{}", closing connection"#,
                                socket_addr,
                                String::from_utf8_lossy(message.as_slice())
                            ),
                            ClientConnectionError::Aborted => {
                                info!(r#"Client {} disconnected from server"#, socket_addr)
                            }
                            ClientConnectionError::ReaderError => info!(
                                r#"Failed to read connection message from client {}, closing connection"#,
                                socket_addr
                            ),
                        }
                        return;
                    }
                };

                match client {
                    Client::Subscriber(mut subscriber) => {
                        info!(
                            r#"For topic "{}" connected subscriber with ip {}"#,
                            &subscriber.topic, &subscriber.socket_addr
                        );
                        match subscriber.listen(&mut stream).await {
                            Ok(_) => info!(
                                r#"Subscriber {} disconnected from topic "{}""#,
                                &subscriber.socket_addr, &subscriber.topic
                            ),
                            Err(e) => error!(
                                r#"Subscriber {} disconnected from topic "{}" with error: {:?}"#,
                                &subscriber.socket_addr, &subscriber.topic, e
                            ),
                        };
                    }
                    Client::Publisher(mut publisher) => {
                        info!(
                            r#"For topic "{}" connected publisher with ip {}"#,
                            &publisher.topic, &publisher.socket_addr
                        );
                        match publisher.listen(&mut buf_reader).await {
                            Ok(_) => info!(
                                r#"Publisher {} disconnected from topic "{}""#,
                                &publisher.socket_addr, &publisher.topic
                            ),
                            Err(e) => error!(
                                r#"Publisher {} disconnected from topic "{}" with error: {:?}"#,
                                &publisher.socket_addr, &publisher.topic, e
                            ),
                        };
                    }
                }
            });
        }
    }
}
