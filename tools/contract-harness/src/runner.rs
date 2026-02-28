//! HTTP request runner — sends one fixture request and captures the response.

use reqwest::Client;

use crate::fixture::Fixture;

/// Result of running a single fixture assertion.
pub struct RunResult {
    pub expected_status: u16,
    pub actual_status: Option<u16>,
    /// Headers that were expected but missing or had the wrong value.
    pub header_mismatches: Vec<String>,
    /// Set when the request could not be sent (e.g. connection refused).
    pub error: Option<String>,
}

impl RunResult {
    pub fn passed(&self) -> bool {
        self.error.is_none()
            && self.actual_status == Some(self.expected_status)
            && self.header_mismatches.is_empty()
    }
}

pub struct Runner {
    client: Client,
    base_url: String,
}

impl Runner {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_owned(),
        }
    }

    pub async fn run(&self, fixture: &Fixture) -> RunResult {
        let url = format!("{}{}", self.base_url, fixture.request.path);

        let method =
            match reqwest::Method::from_bytes(fixture.request.method.to_uppercase().as_bytes()) {
                Ok(m) => m,
                Err(_) => {
                    return RunResult {
                        expected_status: fixture.expect.status,
                        actual_status: None,
                        header_mismatches: Vec::new(),
                        error: Some(format!("unknown HTTP method: {}", fixture.request.method)),
                    };
                }
            };

        let mut req = self.client.request(method, &url);
        for (k, v) in &fixture.request.headers {
            req = req.header(k, v);
        }
        if let Some(body) = &fixture.request.body {
            req = req.json(body);
        }

        match req.send().await {
            Ok(resp) => {
                let actual_status = resp.status().as_u16();
                let headers = resp.headers().clone();

                // Check expected headers (subset match).
                let mut header_mismatches = Vec::new();
                for (name, expected_val) in &fixture.expect.headers {
                    match headers.get(name.as_str()) {
                        Some(actual_val) if actual_val.to_str().unwrap_or("") == expected_val => {}
                        Some(actual_val) => {
                            header_mismatches.push(format!(
                                "{name}: expected {:?}, got {:?}",
                                expected_val,
                                actual_val.to_str().unwrap_or("<non-utf8>")
                            ));
                        }
                        None => {
                            header_mismatches
                                .push(format!("{name}: missing (expected {expected_val:?})"));
                        }
                    }
                }

                RunResult {
                    expected_status: fixture.expect.status,
                    actual_status: Some(actual_status),
                    header_mismatches,
                    error: None,
                }
            }
            Err(e) => RunResult {
                expected_status: fixture.expect.status,
                actual_status: None,
                header_mismatches: Vec::new(),
                error: Some(e.to_string()),
            },
        }
    }
}
