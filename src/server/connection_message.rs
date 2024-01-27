pub use crate::server::client::ClientType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
/// The first message from clients after connecting to the `Server`
///
/// Each line is a message from a `Client`
///
/// The first message must be provided in JSON format in the following format:
///
/// ```JSON
/// {"method": "<client_type>", "topic": "<topic_name>"}
/// ```
///
/// - Where `<client_type>` is the serialized [ClientType](ClientType)
/// - `<topic_name>` - a string describing which topic the client is connecting to
///
/// It is currently not possible to change the settings that were described in the first message
pub struct ConnectionMessage {
    #[serde(rename = "method")]
    pub client_type: ClientType,
    pub topic: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::client::ClientType::{Publisher, Subscriber};

    #[test]
    fn test_format() {
        assert_eq!(
            serde_json::from_str::<ConnectionMessage>(
                r#"{"method": "subscribe", "topic": "Topic"}"#
            )
            .unwrap(),
            ConnectionMessage {
                client_type: Subscriber,
                topic: "Topic".to_string()
            }
        );

        assert_eq!(
            serde_json::from_str::<ConnectionMessage>(r#"{"method": "publish", "topic": ""}"#)
                .unwrap(),
            ConnectionMessage {
                client_type: Publisher,
                topic: "".to_string()
            }
        );

        assert_eq!(
            serde_json::from_str::<ConnectionMessage>(
                r#" { "method" :  "publish" ,  "topic":  ""  }  "#
            )
            .unwrap(),
            ConnectionMessage {
                client_type: Publisher,
                topic: "".to_string()
            }
        );
    }
}
