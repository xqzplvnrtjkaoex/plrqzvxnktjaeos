//! Domain types shared across all Madome services.
//!
//! This crate contains only pure types with no framework dependencies.
//! Import in `usecase/` and `domain/` layers; never in `infra/` or `handlers/`.

pub mod activity;
pub mod book;
pub mod book_tag;
pub mod id;
pub mod pagination;
pub mod user;
