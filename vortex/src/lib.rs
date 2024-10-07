//! # Vortex
#![feature(negative_impls)]
// Start of code
#![deny(clippy::correctness, clippy::complexity, clippy::all)]
#![warn(clippy::perf, clippy::suspicious, clippy::style)]

pub mod build_constants;
pub mod debug;
pub mod entry;
pub mod event;
pub mod layer;
pub mod log;
pub mod render;
pub mod window;
