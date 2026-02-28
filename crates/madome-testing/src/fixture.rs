//! Contract fixture loader.
//!
//! Loads golden files from `contracts/http/` for contract assertion tests.

use std::path::Path;

use serde_json::Value;

/// Load a JSON fixture file relative to the workspace root.
///
/// # Example
/// ```no_run
/// use madome_testing::fixture::Fixture;
/// let val = Fixture::load("contracts/http/auth/create_token.json");
/// ```
pub struct Fixture;

impl Fixture {
    /// Load and parse a fixture JSON file at `workspace_root/path`.
    ///
    /// Panics if the file is missing or invalid JSON.
    pub fn load(relative_path: &str) -> Value {
        let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
            .map(|dir| {
                // Walk up from crate dir to workspace root
                let p = Path::new(&dir);
                p.ancestors()
                    .find(|a| a.join("Cargo.lock").exists())
                    .unwrap_or(p)
                    .to_path_buf()
            })
            .unwrap_or_else(|_| std::env::current_dir().unwrap());

        let full_path = workspace_root.join(relative_path);
        let contents = std::fs::read_to_string(&full_path)
            .unwrap_or_else(|e| panic!("fixture not found at {}: {}", full_path.display(), e));
        serde_json::from_str(&contents)
            .unwrap_or_else(|e| panic!("invalid JSON in fixture {}: {}", relative_path, e))
    }
}
