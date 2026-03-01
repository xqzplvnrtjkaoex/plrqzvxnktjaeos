//! Test result reporter — formats PASS/FAIL output and prints a summary.

use crate::{fixture::Fixture, runner::RunResult};

pub struct Reporter {
    passed: usize,
    failed: usize,
}

impl Default for Reporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter {
    pub fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
        }
    }

    pub fn record(&mut self, fixture: &Fixture, result: RunResult) {
        if result.passed() {
            self.passed += 1;
            println!(
                "PASS  [{}/{}] {}",
                fixture.service, fixture.id, fixture.description
            );
        } else {
            self.failed += 1;
            println!(
                "FAIL  [{}/{}] {}",
                fixture.service, fixture.id, fixture.description
            );
            if let Some(err) = &result.error {
                println!("        error: {err}");
            } else if let Some(actual) = result.actual_status {
                if actual != result.expected_status {
                    println!(
                        "        {} {} → expected {}, got {}",
                        fixture.request.method,
                        fixture.request.path,
                        result.expected_status,
                        actual
                    );
                }
                for mismatch in &result.header_mismatches {
                    println!("        header: {mismatch}");
                }
                if let Some(mismatch) = &result.body_mismatch {
                    println!("        {mismatch}");
                }
            }
        }
    }

    pub fn print_summary(&self) {
        println!();
        println!("────────────────────────────────────────────────────");
        println!("Results: {} passed, {} failed", self.passed, self.failed);
    }

    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}
