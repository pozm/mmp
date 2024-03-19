use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongEntry {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: u64,
    pub sample_rate: u32,
    pub bit_rate: u32,
    pub bit_depth: u16,
    pub channels: u16,
    pub year: u16,
    pub metadata: HashMap<String, String>,
}
