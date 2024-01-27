use super::utils::current_time;
use crate::message::MessageQueue;
use serde_json::json;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::timeout;

const GL_EXTRA_TIMEOUT: f64 = 2.0;

pub struct Subscriber {
    buf_reader: BufReader<TcpStream>,
}

impl Subscriber {
    pub async fn new(server_address: &SocketAddr) -> Self {
        let stream = TcpStream::connect(server_address)
            .await
            .expect(format!("Cannot connect to server {}", server_address).as_str());
        Self {
            buf_reader: BufReader::new(stream),
        }
    }

    async fn auth(&mut self, topic: String, delimiter: u8) -> bool {
        let mut message_bytes: Vec<u8> = json!({
            "method": "subscribe",
            "topic": topic
        })
        .to_string()
        .as_bytes()
        .to_vec();
        message_bytes.push(delimiter);
        if self
            .buf_reader
            .write(message_bytes.as_slice())
            .await
            .is_err()
        {
            return false;
        }
        true
    }

    pub async fn wait_message(&mut self) -> bool {
        let mut buf = [0u8; 1];
        match self.buf_reader.read(&mut buf).await {
            Ok(sz) => sz > 0usize,
            Err(_) => false,
        }
    }

    pub async fn read_message(&mut self, delimiter: u8) -> Option<String> {
        let mut buffer: Vec<u8> = vec![];

        match self.buf_reader.read_until(delimiter, &mut buffer).await {
            Ok(_) => Some(String::from_utf8_lossy(buffer.as_slice()).into_owned()),
            Err(_) => None,
        }
    }
}

pub struct SubscriberTester {
    subscriber: Subscriber,
    delimiter: u8,
}

impl SubscriberTester {
    pub fn new(subscriber: Subscriber, delimiter: u8) -> Self {
        Self {
            subscriber,
            delimiter,
        }
    }

    pub async fn test_messages(
        &mut self,
        topic: String,
        mut message_queue: MessageQueue,
        after_all_timeout: f64, /* in seconds */
    ) -> bool {
        if !self.subscriber.auth(topic, self.delimiter).await {
            return false;
        }

        while !message_queue.is_empty() {
            let timout_sec = message_queue.start_time
                + GL_EXTRA_TIMEOUT
                + message_queue.current_message().expected_sent_time
                - current_time();

            match timeout(
                Duration::from_secs_f64(timout_sec),
                self.subscriber.read_message(self.delimiter),
            )
            .await
            {
                Ok(message) => {
                    if message.is_none()
                        || !message_queue.new_message(message.unwrap().strip_suffix("\n").unwrap())
                    {
                        return false;
                    }
                }
                Err(_) => return false,
            }
        }

        timeout(
            Duration::from_secs_f64(after_all_timeout),
            self.subscriber.wait_message(),
        )
        .await
        .map_or(true, |has_message| !has_message)
    }
}
