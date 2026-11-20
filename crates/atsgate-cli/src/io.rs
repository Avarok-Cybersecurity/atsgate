//! The I/O seam for inputs: files and HTTPS. Everything that decides what to do with the bytes
//! takes `&dyn Io`, so tests can supply bytes without touching the disk or the network.

use std::path::Path;
use std::time::Duration;

pub trait Io {
    fn read(&self, path: &Path) -> Result<Vec<u8>, String>;
    /// GETs an `https://` URL, failing when the body exceeds `max_bytes`.
    fn fetch(&self, url: &str, max_bytes: u64) -> Result<Vec<u8>, String>;
}

/// The real file system and network.
#[derive(Debug)]
pub struct System {
    /// Whole-request timeout for a fetch.
    pub timeout: Duration,
}

impl Io for System {
    fn read(&self, path: &Path) -> Result<Vec<u8>, String> {
        std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
    }

    fn fetch(&self, url: &str, max_bytes: u64) -> Result<Vec<u8>, String> {
        if !url.starts_with("https://") {
            return Err(format!("{url}: only https:// URLs are fetched"));
        }
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .https_only(true)
            .timeout_global(Some(self.timeout))
            .build()
            .into();
        let mut response = agent.get(url).call().map_err(|e| format!("{url}: {e}"))?;
        response
            .body_mut()
            .with_config()
            .limit(max_bytes)
            .read_to_vec()
            .map_err(|e| format!("{url}: {e} (limit {max_bytes} bytes)"))
    }
}
