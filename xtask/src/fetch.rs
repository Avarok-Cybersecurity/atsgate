//! Downloads into the gitignored `.cache/` directory, verified against pinned sha256 digests.
//! This module is the only place xtask touches the network; it shells out to `curl`.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn cache_dir(parts: &[&str]) -> PathBuf {
    let mut dir = crate::root().join(".cache");
    dir.extend(parts);
    dir
}

/// Fetches `url` into `path` unless a cached copy exists. No digest check (see [`pinned`]).
pub fn cached(url: &str, path: &Path) -> Result<Vec<u8>, String> {
    if let Ok(bytes) = std::fs::read(path)
        && !bytes.is_empty()
    {
        return Ok(bytes);
    }
    let parent = path.parent().ok_or("cache path has no parent")?;
    std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    let part = path.with_extension("part");
    let status = Command::new("curl")
        .args(["-sSfL", "--retry", "5", "--retry-delay", "10", "-m", "900"])
        .args([
            "-A",
            "atsgate-xtask (+https://github.com/Avarok-Cybersecurity/atsgate)",
        ])
        .arg("-o")
        .arg(&part)
        .arg(url)
        .status()
        .map_err(|e| format!("curl: {e}"))?;
    if !status.success() {
        return Err(format!("curl failed for {url}: {status}"));
    }
    std::fs::rename(&part, path).map_err(|e| format!("{}: {e}", path.display()))?;
    std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Fetches `url` (cached) and fails unless its bytes hash to `sha256`.
pub fn pinned(url: &str, sha256: &str, path: &Path) -> Result<Vec<u8>, String> {
    let bytes = cached(url, path)?;
    let actual = sha256_hex(&bytes);
    if actual != sha256 {
        return Err(format!(
            "{url}: sha256 {actual} does not match the pin {sha256}; delete {} to refetch",
            path.display()
        ));
    }
    Ok(bytes)
}

/// Fetches many URLs into their cache paths, four at a time.
pub fn cached_all(jobs: &[(String, PathBuf)]) -> Result<Vec<Vec<u8>>, String> {
    let mut out: Vec<Option<Vec<u8>>> = vec![None; jobs.len()];
    for (chunk_index, chunk) in jobs.chunks(4).enumerate() {
        let results: Vec<Result<Vec<u8>, String>> = std::thread::scope(|s| {
            let handles: Vec<_> = chunk
                .iter()
                .map(|(url, path)| s.spawn(move || cached(url, path)))
                .collect();
            handles
                .into_iter()
                .map(|h| {
                    h.join()
                        .unwrap_or_else(|_| Err("fetch thread panicked".into()))
                })
                .collect()
        });
        for (i, r) in results.into_iter().enumerate() {
            out[chunk_index * 4 + i] = Some(r?);
        }
    }
    Ok(out.into_iter().flatten().collect())
}
