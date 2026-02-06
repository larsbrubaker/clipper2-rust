# Clipper2 Rust

A Polygon <a href="https://en.wikipedia.org/wiki/Clipping_(computer_graphics)">Clipping</a>, <a href="https://en.wikipedia.org/wiki/Parallel_curve">Offsetting</a> and <a href="https://en.wikipedia.org/wiki/Constrained_Delaunay_triangulation">Triangulation</a> library — a pure Rust port of the [Clipper2 C++ library](https://github.com/AngusJohnson/Clipper2) by [Angus Johnson](https://www.angusj.com).

[![License](https://img.shields.io/badge/License-Boost_1.0-lightblue.svg)](https://www.boost.org/LICENSE_1_0.txt)
[![documentation](https://user-images.githubusercontent.com/5280692/187832279-b2a43890-da80-4888-95fe-793f092be372.svg)](https://www.angusj.com/clipper2/Docs/Overview.htm)

## Overview

The **Clipper2** library performs **intersection**, **union**, **difference** and **XOR** boolean operations on both simple and complex polygons. It also performs polygon **offsetting/inflating** and **Constrained Delaunay Triangulation**.

This is a pure Rust port of the original [Clipper2 C++ library](https://github.com/AngusJohnson/Clipper2), aiming for identical functionality, algorithmic behavior, and performance characteristics while leveraging Rust's memory safety guarantees and modern language features.

This port was created by [MatterHackers](https://www.matterhackers.com) using [Claude](https://www.anthropic.com/claude) by Anthropic.

## Features

- **Boolean Operations**: Intersection, Union, Difference, XOR on both simple and complex polygons
- **Polygon Offsetting**: Inflate/deflate polygons with various join types
- **Constrained Delaunay Triangulation**: Triangulate polygons with constraints
- **Rectangle Clipping**: High-performance rectangular clipping
- **Path Simplification**: Reduce polygon complexity while preserving shape
- **Multiple Precision**: Support for both integer (`i64`) and floating-point (`f64`) coordinates
- **PolyTree Structure**: Hierarchical representation of polygon relationships
- **Memory Safe**: All the benefits of Rust's ownership system with zero-cost abstractions

## Visual Examples

**Clipping**

![clipperB](https://user-images.githubusercontent.com/5280692/178123810-1719a1f5-25c3-4a9e-b419-e575ff056272.svg)

**Inflating (aka Offsetting)**

![rabbit](https://github.com/user-attachments/assets/a0f2f43c-f0a3-45ec-887d-d9ca34256088)
![rabbit_offset](https://github.com/user-attachments/assets/ca05688e-293f-4596-86ab-df529694e778)

**Constrained Delaunay Triangulation**

![coral3](https://github.com/user-attachments/assets/78e88382-f772-442b-a09c-c14d8906fb21)
![coral3t](https://github.com/user-attachments/assets/c329ef2a-4833-4092-8415-145400fba8b0)

![coral3_t2](https://github.com/user-attachments/assets/fc1c8741-e033-4dc6-869a-c0d7da550cfa)

## Documentation

Comprehensive documentation for the Clipper2 algorithms and API is available at:

<a href="https://www.angusj.com/clipper2/Docs/Overview.htm"><b>Clipper2 Documentation</b></a>

The Rust API follows the same structure and naming conventions (adapted to Rust idioms) as the original C++ library, so the upstream documentation serves as an excellent reference.

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
clipper2 = "0.1.0"
```

### Basic Usage

```rust
use clipper2::*;

// Create subject and clip polygons
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
git clone https://github.com/MatterHackers/clipper2-rust.git
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

This project follows strict implementation rules to ensure fidelity with the original C++ library:

1. **Exact Behavioral Matching**: The Rust implementation must match C++ behavior exactly — same algorithms, same mathematical precision, same edge case handling
2. **No Stubs**: Every function must be complete and production-ready
3. **Dependency-Driven**: Functions are only implemented when all dependencies are ready
4. **Comprehensive Testing**: Tests verify exact behavioral match with the C++ implementation

See [CLAUDE.md](CLAUDE.md) for complete implementation guidelines.

## Contributing

Contributions are welcome! Please note the strict implementation requirements:

1. All functions must be **complete** — no `todo!()` or stubs
2. Comprehensive tests required for all functionality
3. Must exactly match C++ behavioral semantics
4. Follow the dependency-driven implementation order

## License

This project is licensed under the [Boost Software License 1.0](https://www.boost.org/LICENSE_1_0.txt), the same license as the original Clipper2 C++ library.

## Acknowledgments

- **[Angus Johnson](https://www.angusj.com)** — Author of the original [Clipper2](https://github.com/AngusJohnson/Clipper2) library
- **[MatterHackers](https://www.matterhackers.com)** — Developed this Rust port
- **[Claude](https://www.anthropic.com/claude) by Anthropic** — AI assistant used to perform the port
