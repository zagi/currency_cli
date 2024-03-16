use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Serialize, Deserialize)]
pub struct Rates {
    pub rates: HashMap<String, f64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CacheItem {
    pub rates: HashMap<String, f64>,
    pub timestamp: SystemTime,
}
