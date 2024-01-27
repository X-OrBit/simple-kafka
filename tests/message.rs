use super::utils::current_time;

#[derive(Clone)]
pub struct Message {
    pub message: String,
    pub expected_sent_time: f64, // unix in seconds
}

impl Message {
    pub fn new(message: String, expected_sent_time: f64) -> Self {
        Self {
            message,
            expected_sent_time,
        }
    }
}

pub type Messages = Vec<Message>;

pub const TIME_THRESHOLD: f64 = 2.0; // seconds

#[derive(Clone)]
pub struct MessageQueue {
    index: usize,
    pub messages: Messages,
    pub start_time: f64, // unix in seconds
}

impl MessageQueue {
    pub fn default() -> Self {
        Self {
            index: 0,
            messages: vec![],
            start_time: current_time(),
        }
    }

    pub fn new(messages: Messages) -> Self {
        Self {
            index: 0,
            messages,
            start_time: current_time(),
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        let mut union_messages = self.messages.clone();
        union_messages.append(&mut other.messages.clone());
        union_messages.sort_by(|m1: &Message, m2: &Message| {
            m1.expected_sent_time
                .partial_cmp(&m2.expected_sent_time)
                .unwrap()
        });

        Self {
            index: 0,
            messages: union_messages,
            start_time: if self.start_time < other.start_time {
                self.start_time
            } else {
                other.start_time
            },
        }
    }

    pub fn push(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn current_message(&self) -> &Message {
        &self.messages[self.index]
    }

    pub fn new_message(&mut self, message: &str) -> bool {
        if self.is_empty() {
            return false;
        }

        let current_time = current_time() - self.start_time;
        if !(self.messages[self.index].expected_sent_time <= current_time
            && current_time <= self.messages[self.index].expected_sent_time + TIME_THRESHOLD)
            || self.messages[self.index].message != message
        {
            return false;
        }

        self.index += 1;
        true
    }

    pub fn is_empty(&self) -> bool {
        self.messages.len() <= self.index
    }
}
