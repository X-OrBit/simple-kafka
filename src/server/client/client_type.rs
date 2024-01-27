use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
/// The type of [Client](crate::server::client::Client) that is served by the `Server`
///
/// In serialized format is it `"subscriber"` (for `Subscriber`s) or `"publish"` (for `Publisher`s)
pub enum ClientType {
    #[serde(rename = "subscribe")]
    Subscriber,
    #[serde(rename = "publish")]
    Publisher,
}
