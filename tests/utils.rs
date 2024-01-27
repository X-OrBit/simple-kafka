use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_time() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64()
}
