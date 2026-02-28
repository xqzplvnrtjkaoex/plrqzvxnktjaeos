//! Mock gRPC server helpers.
//!
//! Provides an in-process tonic server with configurable responses for testing
//! services that call other services via gRPC.
//!
//! Full implementations are added per-service as each Unit is built.
//! This file contains the shared skeleton.

/// Marker trait for mock gRPC service implementations.
///
/// Concrete mock servers for UserService, LibraryService, and NotificationService
/// are added in their respective service test modules (Units D, E, G).
pub trait MockGrpcService: Send + Sync + 'static {}
