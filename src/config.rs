use std::time::Duration;

pub static CACHE_DURATION: Duration = Duration::new(3600, 0); // 1 hour
pub const CACHE_FILE: &str = "cache.json";
