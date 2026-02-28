//! Contract fixture format and loader.
//!
//! Each fixture file at `contracts/http/{service}/{id}.json` describes one HTTP
//! assertion: the request to send and the expected response status.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

/// A single HTTP contract assertion loaded from a fixture file.
#[derive(Debug, Clone, Deserialize)]
pub struct Fixture {
    /// Service name used for filtering (`auth`, `library`, `users`).
    pub service: String,
    /// Unique identifier within the service (matches the filename stem).
    pub id: String,
    /// Human-readable description shown in test output.
    pub description: String,
    pub request: Request,
    pub expect: Expect,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Expect {
    /// Expected HTTP status code.
    pub status: u16,
    /// Expected response headers (subset match — extra headers are allowed).
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

/// Load all fixture files from `{workspace_root}/contracts/http/`, optionally
/// filtered to a single service subdirectory.
pub fn load_all(workspace_root: &Path, service: Option<&str>) -> Result<Vec<Fixture>> {
    let http_dir = workspace_root.join("contracts/http");

    let service_dirs: Vec<_> = match service {
        Some(svc) => vec![http_dir.join(svc)],
        None => fs::read_dir(&http_dir)
            .with_context(|| format!("cannot open {}", http_dir.display()))?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .map(|e| e.path())
            .collect(),
    };

    let mut fixtures = Vec::new();
    for dir in service_dirs {
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir)
            .with_context(|| format!("cannot read {}", dir.display()))?
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("cannot read {}", path.display()))?;
                let fixture: Fixture = serde_json::from_str(&content)
                    .with_context(|| format!("invalid fixture JSON in {}", path.display()))?;
                fixtures.push(fixture);
            }
        }
    }

    fixtures.sort_by(|a, b| a.service.cmp(&b.service).then(a.id.cmp(&b.id)));
    Ok(fixtures)
}
