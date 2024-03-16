use crate::config::CACHE_FILE;
use crate::models::CacheItem;
use serde_json;
use std::{
    collections::HashMap,
    fs::File,
    io,
    io::{BufReader, BufWriter},
};

pub fn save_cache(cache: &HashMap<String, CacheItem>) -> Result<(), io::Error> {
    let file = File::create(CACHE_FILE)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, cache)?;
    Ok(())
}

pub fn load_cache() -> Result<HashMap<String, CacheItem>, io::Error> {
    if let Ok(file) = File::open(CACHE_FILE) {
        let reader = BufReader::new(file);
        let cache = serde_json::from_reader(reader)?;
        Ok(cache)
    } else {
        Ok(HashMap::new())
    }
}
