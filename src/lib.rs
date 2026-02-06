//! # Clipper2 - High-performance 2D polygon clipping library
//!
//! This is a complete Rust port of the Clipper2 C++ library by Angus Johnson.
//!
//! ## Implementation Status
//!
//! Following STRICT implementation rules:
//! - NO stubs or todo macros allowed
//! - Functions implemented only when all dependencies are ready
//! - Each function must have comprehensive tests
//! - Exact behavioral matching with C++ implementation
//!
//! Total items to implement: 857 (790 functions + 56 classes + 11 enums)

// Module structure mirrors C++ header organization
// Following STRICT RULES - only include implemented modules
pub mod core;
pub mod engine;
pub mod engine_fns;
pub mod engine_public;
pub mod rectclip;
pub mod version;

// Future modules to implement in dependency order (NO STUBS ALLOWED):
// pub mod offset;   // clipper.offset.h - Path offsetting
// pub mod minkowski; // clipper.minkowski.h - Minkowski operations
// pub mod export;   // clipper.export.h - Export utilities

// Re-export implemented types and functions only
pub use core::*;
pub use engine::*;
pub use engine_fns::*;
pub use engine_public::*;
pub use rectclip::*;
pub use version::*;
