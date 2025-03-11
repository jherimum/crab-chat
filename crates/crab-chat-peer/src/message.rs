use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Builder, Clone, Getters)]
pub struct Message {
    data: String,
    timestamp: u64,
    topic: String,
}
