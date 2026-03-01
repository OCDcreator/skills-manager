use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::Path;
use walkdir::WalkDir;

const IGNORED: &[&str] = &[".git", ".DS_Store", "Thumbs.db", ".gitignore"];

pub fn hash_directory(dir: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut entries: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !IGNORED.contains(&name.as_ref())
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    entries.sort_by(|a, b| a.path().cmp(b.path()));

    for entry in entries {
        let rel = entry
            .path()
            .strip_prefix(dir)
            .unwrap_or(entry.path())
            .to_string_lossy();
        hasher.update(rel.as_bytes());
        if let Ok(content) = std::fs::read(entry.path()) {
            hasher.update(&content);
        }
    }

    Ok(hex::encode(hasher.finalize()))
}
