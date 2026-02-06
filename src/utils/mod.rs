// Copyright 2025 - Clipper2 Rust port
// Utility modules for visualization, file I/O, colors, and timing
//
// These correspond to the C++ files in CPP/Utils/:
// - clipper.svg.h / clipper.svg.cpp -> svg.rs
// - clipper.svg.utils.h -> svg.rs (utility functions)
// - ClipFileLoad.h / ClipFileLoad.cpp -> file_io.rs
// - ClipFileSave.h / ClipFileSave.cpp -> file_io.rs
// - Colors.h -> colors.rs
// - Timer.h -> timer.rs

pub mod colors;
pub mod file_io;
pub mod svg;
pub mod timer;
