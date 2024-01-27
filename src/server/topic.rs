use tokio::sync::broadcast::{channel, Sender};

const CHANNEL_BUFFER_SIZE: usize = 32usize;

pub(crate) struct Topic {
    pub name: String,
    pub sender: Sender<Vec<u8>>,
}

impl Clone for Topic {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            sender: self.sender.clone(),
        }
    }
}

impl Topic {
    pub async fn new(topic_name: String) -> Self {
        let (sender, _) = channel(CHANNEL_BUFFER_SIZE);
        Self {
            name: topic_name,
            sender,
        }
    }
}
