pub use crate::server::client::client_type::ClientType;
pub use crate::server::client::{publisher::Publisher, subscriber::Subscriber};

pub mod client_type;
pub mod publisher;
pub mod server_broker;
pub mod subscriber;

#[derive(Debug)]
/// While sending a Connection Message, an error may occur.
/// They are described in this structure
pub enum ClientConnectionError {
    /// The sent message cannot be deserialized or it is not in the [provided](crate::server::connection_message::ConnectionMessage) format
    UnexpectedMessage(Vec<u8>),
    /// Connection aborted
    Aborted,
    /// Some reader error
    ReaderError,
}

/// `Client` is `Server`'s client
///
/// Can be [Subscriber](Subscriber) or [Publisher](Publisher)
///
/// To introduce yourself to the server and state which `ClientType` the user is and which topic he wants to connect to,
/// he must send to server a [ConnectionMessage](crate::server::connection_message::ConnectionMessage)
pub enum Client {
    Subscriber(Subscriber),
    Publisher(Publisher),
}
