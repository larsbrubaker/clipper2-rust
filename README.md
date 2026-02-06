# Clipper2 Rust Port

A high-performance 2D polygon clipping library - Rust port of the [Clipper2 C++ library](https://github.com/AngusJohnson/Clipper2) by Angus Johnson.

[![Crates.io](https://img.shields.io/crates/v/clipper2.svg)](https://crates.io/crates/clipper2)
[![Documentation](https://docs.rs/clipper2/badge.svg)](https://docs.rs/clipper2)
[![License: BSL-1.0](https://img.shields.io/badge/License-BSL--1.0-blue.svg)](https://opensource.org/licenses/BSL-1.0)

## Overview

Clipper2 is a comprehensive 2D polygon clipping library that performs **intersection**, **union**, **difference** and **XOR** boolean operations on polygons. It also performs polygon **offsetting/inflating** and **path simplification**.

This Rust port aims to provide identical functionality and performance characteristics to the original C++ implementation while leveraging Rust's memory safety and modern language features.

## Features

- **Boolean Operations**: Intersection, Union, Difference, XOR
- **Polygon Offsetting**: Inflate/deflate polygons with various join types
- **Rectangle Clipping**: High-performance rectangular clipping
- **Path Simplification**: Reduce polygon complexity while preserving shape
- **Multiple Precision**: Support for both integer (`i64`) and floating-point (`f64`) coordinates
- **PolyTree Structure**: Hierarchical representation of polygon relationships
- **Memory Safe**: All the benefits of Rust's ownership system

## Implementation Status

🚧 **This project is currently under active development** 🚧

Following a **ZERO TOLERANCE POLICY** for incomplete implementations:
- ✅ **Core Types**: Basic geometric types and operations
- ✅ **Version Info**: Library version constants  
- 🚧 **Clipping Engine**: Main boolean operations engine
- 🚧 **Polygon Offsetting**: Path inflation/deflation
- 🚧 **Rectangle Clipping**: Optimized rectangular clipping
- 🚧 **Minkowski Operations**: Advanced geometric operations
- 🚧 **Export Utilities**: Data export functionality

See [CLAUDE.md](CLAUDE.md) for implementation guidelines.

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
clipper2 = "0.1.0"
```

### Basic Usage

```rust
use clipper2::*;

// Create some polygons
let subject = vec![
    Point64::new(100, 100),
    Point64::new(300, 100), 
    Point64::new(300, 300),
    Point64::new(100, 300),
];

let clip = vec![
    Point64::new(200, 200),
    Point64::new(400, 200),
    Point64::new(400, 400), 
    Point64::new(200, 400),
];

// Note: Full clipping operations not yet implemented
// This is a preview of the planned API
```

## Architecture

### Coordinate Systems

The library supports two coordinate systems:

- **`Path64`/`Paths64`**: Integer coordinates using `i64` 
- **`PathD`/`PathsD`**: Floating-point coordinates using `f64`

### Key Types

```rust
// Points
pub struct Point64 { pub x: i64, pub y: i64 }
pub struct PointD { pub x: f64, pub y: f64 }

// Paths (polygons)
pub type Path64 = Vec<Point64>;
pub type PathD = Vec<PointD>;
pub type Paths64 = Vec<Path64>;
pub type PathsD = Vec<PathD>;

// Rectangles
pub struct Rect64 { /* ... */ }
pub struct RectD { /* ... */ }
```

## Development

### Prerequisites

- Rust 1.70+ (2021 edition)
- Git

### Building

```bash
git clone https://github.com/your-username/clipper2-rust.git
cd clipper2-rust
cargo build
```

### Testing

```bash
cargo test
```

### Benchmarking

```bash
cargo bench
```

## Implementation Philosophy

This project follows **extremely strict implementation rules**:

1. **No Stubs**: Every function must be complete and production-ready
2. **Dependency-Driven**: Functions only implemented when all dependencies are ready
3. **Comprehensive Testing**: 100% test coverage with exact C++ behavioral matching
4. **Zero Tolerance**: No compromises, shortcuts, or "good enough" implementations

See [CLAUDE.md](CLAUDE.md) for complete implementation guidelines.

## Performance

Target performance characteristics:
- Match or exceed C++ Clipper2 performance
- Leverage Rust's zero-cost abstractions
- Optimize for modern CPU architectures
- Comprehensive benchmarking against reference implementation

## Contributing

Contributions are welcome! However, please note the strict implementation requirements:

1. All functions must be **complete** - no `todo!()` or stubs
2. Comprehensive tests required before marking functions as implemented
3. Must exactly match C++ behavioral semantics
4. Follow the dependency-driven implementation order

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## License

This project is licensed under the Boost Software License 1.0 - see the [LICENSE](LICENSE) file for details.

This is the same license as the original Clipper2 C++ library.

## Acknowledgments

- **Angus Johnson** - Original Clipper2 C++ library author
- **Clipper2 Community** - Testing, feedback, and contributions to the original library

## Related Projects

- [Clipper2 C++](https://github.com/AngusJohnson/Clipper2) - Original implementation
- [clipper-sys](https://crates.io/crates/clipper-sys) - Rust bindings to C++ Clipper
- [geo](https://crates.io/crates/geo) - Rust geospatial primitives and algorithms

## Status Updates

For the latest implementation status and progress updates, see:
- [Issues](https://github.com/your-username/clipper2-rust/issues) - Bug reports and feature requests
- [Discussions](https://github.com/your-username/clipper2-rust/discussions) - Community discussions

---

**Note**: This library is currently in early development. APIs may change before 1.0 release.