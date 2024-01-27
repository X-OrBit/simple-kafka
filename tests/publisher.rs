use crate::message::MessageQueue;
use serde_json::json;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub struct Publisher {
    stream: TcpStream,
}

impl Publisher {
    pub async fn new(server_address: &SocketAddr) -> Self {
        Self {
            stream: TcpStream::connect(server_address)
                .await
                .expect(format!("Cannot connect to server {}", server_address).as_str()),
        }
    }

    async fn auth(&mut self, topic: String, delimiter: u8) -> bool {
        let mut message_bytes: Vec<u8> = json!({
            "method": "publish",
            "topic": topic
        })
        .to_string()
        .as_bytes()
        .to_vec();
        message_bytes.push(delimiter);
        if self.stream.write(message_bytes.as_slice()).await.is_err() {
            return false;
        }
        true
    }

    pub async fn send_messages(
        &mut self,
        topic: String,
        message_queue: MessageQueue,
        delimiter: u8,
    ) -> bool {
        if !self.auth(topic, delimiter).await {
            return false;
        }

        let mut previous_expected_sent_time: f64 = 0.0;
        for message in &message_queue.messages {
            tokio::time::sleep(Duration::from_secs_f64(
                message.expected_sent_time - previous_expected_sent_time,
            ))
            .await;
            let mut message_bytes = message.message.as_bytes().to_vec();
            message_bytes.push(delimiter);
            if self.stream.write(message_bytes.as_slice()).await.is_err() {
                return false;
            }
            previous_expected_sent_time = message.expected_sent_time;
        }
        true
    }
}
