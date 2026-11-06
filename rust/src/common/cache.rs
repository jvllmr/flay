use std::{
    cell::RefCell,
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    rc::Rc,
    time::SystemTime,
};

use ruff_python_ast::ModModule;
use ruff_python_parser::{Parsed, parse_module};

use super::fs_cache::hash_parts;

/// A module that was read and parsed once. Reused for every traversal of the
/// same unchanged file, so repeated traversals neither re-read nor re-parse it.
pub struct CacheEntry {
    modified: SystemTime,
    len: u64,
    /// content hash of the source, used as input for the persistent fs cache
    pub hash: String,
    pub source: String,
    pub parsed: Rc<Parsed<ModModule>>,
}

thread_local! {
    static MODULE_CACHE: RefCell<HashMap<PathBuf, Rc<CacheEntry>>> = RefCell::new(HashMap::new());
}

/// Read and parse a module, reusing the cached result when the file did not
/// change since it was last processed (same mtime and size).
pub fn load_module(path: &Path) -> io::Result<Rc<CacheEntry>> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    let len = metadata.len();
    MODULE_CACHE.with(|cache| {
        if let Some(entry) = cache.borrow().get(path)
            && entry.modified == modified
            && entry.len == len
        {
            return Ok(entry.clone());
        }
        let source = fs::read_to_string(path)?;
        let parsed = Rc::new(parse_module(&source).unwrap());
        let hash = hash_parts(&[source.as_bytes()]);
        let entry = Rc::new(CacheEntry {
            modified,
            len,
            hash,
            source,
            parsed,
        });
        cache.borrow_mut().insert(path.to_owned(), entry.clone());
        Ok(entry)
    })
}

pub fn clear_module_cache() {
    MODULE_CACHE.with(|cache| cache.borrow_mut().clear());
}

pub fn module_cache_len() -> usize {
    MODULE_CACHE.with(|cache| cache.borrow().len())
}
