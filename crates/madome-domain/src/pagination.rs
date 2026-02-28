//! Pagination and sort direction types.

use serde::{Deserialize, Serialize};

/// Generic sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Sort {
    Desc,
    Asc,
}

/// Pagination parameters shared across all list endpoints.
///
/// - `per_page`: 1–100, default 25
/// - `page`: ≥ 1, default 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageRequest {
    #[serde(default = "default_per_page", rename = "per-page")]
    pub per_page: u32,
    #[serde(default = "default_page")]
    pub page: u32,
}

fn default_per_page() -> u32 {
    25
}

fn default_page() -> u32 {
    1
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            per_page: default_per_page(),
            page: default_page(),
        }
    }
}

impl PageRequest {
    /// Clamp `per_page` to the valid range 1–100 and `page` to ≥ 1.
    ///
    /// Call after deserializing from query params to enforce bounds.
    pub fn clamped(self) -> Self {
        Self {
            per_page: self.per_page.clamp(1, 100),
            page: self.page.max(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_default_to_per_page_25_page_1() {
        let p = PageRequest::default();
        assert_eq!(p.per_page, 25);
        assert_eq!(p.page, 1);
    }

    #[test]
    fn should_deserialize_defaults_when_fields_absent() {
        let p: PageRequest = serde_json::from_str("{}").unwrap();
        assert_eq!(p.per_page, 25);
        assert_eq!(p.page, 1);
    }

    #[test]
    fn should_clamp_per_page_to_1_100() {
        assert_eq!(
            PageRequest {
                per_page: 0,
                page: 1
            }
            .clamped()
            .per_page,
            1
        );
        assert_eq!(
            PageRequest {
                per_page: 200,
                page: 1
            }
            .clamped()
            .per_page,
            100
        );
        assert_eq!(
            PageRequest {
                per_page: 50,
                page: 1
            }
            .clamped()
            .per_page,
            50
        );
    }

    #[test]
    fn should_clamp_page_to_minimum_1() {
        assert_eq!(
            PageRequest {
                per_page: 25,
                page: 0
            }
            .clamped()
            .page,
            1
        );
        assert_eq!(
            PageRequest {
                per_page: 25,
                page: 5
            }
            .clamped()
            .page,
            5
        );
    }

    #[test]
    fn should_serialize_sort_as_kebab_case() {
        assert_eq!(serde_json::to_string(&Sort::Desc).unwrap(), "\"desc\"");
        assert_eq!(serde_json::to_string(&Sort::Asc).unwrap(), "\"asc\"");
    }
}
