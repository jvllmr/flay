use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use pyo3::prelude::*;
use pyo3::types::PyModule;
use sha2::{Digest, Sha256};

/// Bump when the payload format or the key inputs change, so entries written
/// by older versions never collide with new ones
pub const CACHE_FORMAT_VERSION: &str = "1";

thread_local! {
    static PLATFORM_CACHE_DIR: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static STATS: Cell<(u64, u64)> = const { Cell::new((0, 0)) };
}

/// Directory of the persistent, content-addressed cache. FLAY_CACHE_DIR wins
/// (used by tests); otherwise the platformdirs user cache dir of flay.
pub fn cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("FLAY_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    PLATFORM_CACHE_DIR.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = Some(
                Python::attach(|py| -> PyResult<PathBuf> {
                    let platformdirs = PyModule::import(py, "platformdirs")?;
                    let dir: String = platformdirs
                        .getattr("user_cache_dir")?
                        .call1(("flay",))?
                        .extract()?;
                    Ok(PathBuf::from(dir).join("cache"))
                })
                .unwrap_or_else(|_| std::env::temp_dir().join("flay-cache")),
            );
        }
        slot.clone().unwrap()
    })
}

/// Stable sha256 hex over length-prefixed parts
pub fn hash_parts(parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(CACHE_FORMAT_VERSION.as_bytes());
    for part in parts {
        hasher.update((part.len() as u64).to_le_bytes());
        hasher.update(part);
    }
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn get_counted(key: &str) -> Option<Vec<u8>> {
    let blob = fs::read(cache_dir().join(key)).ok();
    STATS.with(|s| {
        let (hits, misses) = s.get();
        if blob.is_some() {
            s.set((hits + 1, misses));
        } else {
            s.set((hits, misses + 1));
        }
    });
    blob
}

pub fn put(key: &str, payload: &[u8]) {
    let dir = cache_dir();
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let final_path = dir.join(key);
    let tmp_path = dir.join(format!("{key}.{}.tmp", std::process::id()));
    if fs::write(&tmp_path, payload).is_ok() {
        let _ = fs::rename(&tmp_path, final_path);
    }
}

pub fn stats() -> (u64, u64) {
    STATS.with(|s| s.get())
}

pub fn serialize_str_map(map: &HashMap<String, usize>) -> Vec<u8> {
    let mut entries: Vec<(&String, &usize)> = map.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = format!("flay-map-v{CACHE_FORMAT_VERSION}\n").into_bytes();
    for (key, value) in entries {
        out.extend(format!("{key}\t{value}\n").into_bytes());
    }
    out
}

pub fn parse_str_map(blob: &[u8]) -> Option<HashMap<String, usize>> {
    let text = std::str::from_utf8(blob).ok()?;
    let mut lines = text.lines();
    if lines.next()? != format!("flay-map-v{CACHE_FORMAT_VERSION}") {
        return None;
    }
    let mut map = HashMap::new();
    for line in lines {
        let (key, value) = line.split_once('\t')?;
        map.insert(key.to_string(), value.parse().ok()?);
    }
    Some(map)
}
