//! # clipper2-rust
//!
//! A pure Rust port of the [Clipper2](https://github.com/AngusJohnson/Clipper2) polygon
//! clipping and offsetting library by Angus Johnson.
//!
//! ## Features
//!
//! - **Boolean operations**: Intersection, Union, Difference, and XOR on polygons
//! - **Polygon offsetting**: Inflate/deflate with Miter, Square, Bevel, and Round joins
//! - **Rectangle clipping**: High-performance rectangular clipping
//! - **Minkowski sum/difference**: Geometric Minkowski operations
//! - **Path simplification**: Ramer-Douglas-Peucker and Clipper2's simplification
//! - **PolyTree**: Hierarchical parent/child/hole polygon representation
//! - **Dual precision**: Integer (`i64`) and floating-point (`f64`) coordinate support
//!
//! ## Quick Start
//!
//! ```rust
//! use clipper2_rust::core::FillRule;
//!
//! let subject = vec![clipper2_rust::make_path64(&[100, 100, 300, 100, 300, 300, 100, 300])];
//! let clip = vec![clipper2_rust::make_path64(&[200, 200, 400, 200, 400, 400, 200, 400])];
//!
//! let result = clipper2_rust::intersect_64(&subject, &clip, FillRule::NonZero);
//! ```
//!
//! ## Coordinate Systems
//!
//! - [`Path64`] / [`Paths64`]: Integer coordinates (`i64`) — recommended for precision
//! - [`PathD`] / [`PathsD`]: Floating-point coordinates (`f64`) — convenient for external data

pub mod core;
pub mod engine;
pub mod engine_fns;
pub mod engine_public;
pub mod rectclip;
pub mod version;

pub mod clipper;
pub mod minkowski;
pub mod offset;
pub mod utils;

pub use clipper::*;
pub use core::*;
pub use engine::*;
pub use engine_fns::*;
pub use engine_public::*;
pub use minkowski::*;
pub use offset::*;
pub use rectclip::*;
pub use version::*;
