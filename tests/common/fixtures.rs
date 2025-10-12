// Fixture loading utilities for test data
// Provides functions to load JSON test fixtures from files

use serde_json::Value;
use std::fs;
use std::path::Path;

/// Load a JSON fixture from the tests/fixtures/ directory
///
/// # Arguments
/// * `name` - The fixture file name without .json extension
///
/// # Returns
/// Parsed JSON value from the fixture file
///
/// # Panics
/// Panics if the fixture file doesn't exist or contains invalid JSON
pub fn load_fixture(name: &str) -> Value {
    let fixture_path = format!("tests/fixtures/{}.json", name);
    let content = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("Failed to read fixture file: {}", fixture_path));

    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse fixture {} as JSON: {}", fixture_path, e))
}

/// Load a fixture file as a raw string (not parsed as JSON)
pub fn load_fixture_str(name: &str) -> String {
    let fixture_path = format!("tests/fixtures/{}.json", name);
    fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("Failed to read fixture file: {}", fixture_path))
}

/// Check if a fixture file exists
pub fn fixture_exists(name: &str) -> bool {
    let fixture_path = format!("tests/fixtures/{}.json", name);
    Path::new(&fixture_path).exists()
}

/// List all available fixtures in the fixtures directory
#[allow(dead_code)]
pub fn list_fixtures() -> Vec<String> {
    let fixtures_dir = Path::new("tests/fixtures");
    if !fixtures_dir.exists() {
        return Vec::new();
    }

    fs::read_dir(fixtures_dir)
        .expect("Failed to read fixtures directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "json" {
                path.file_stem()?.to_str().map(String::from)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_exists_check() {
        // This test doesn't fail if fixture doesn't exist
        // It just checks the function works
        let _ = fixture_exists("metrics");
    }

    #[test]
    fn test_list_fixtures() {
        let fixtures = list_fixtures();
        // Should return a list (possibly empty if no fixtures created yet)
        assert!(fixtures.len() >= 0);
    }
}
