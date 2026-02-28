//! Test utilities for Madome services.
//!
//! Provides `MockAuthServer`, `TestApp`, fixture loader, and gRPC mock helpers.
//! Import in `#[cfg(test)]` blocks only — never in production code.

pub mod auth;
pub mod fixture;
pub mod grpc;
