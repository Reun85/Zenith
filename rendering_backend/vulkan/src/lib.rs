#![feature(negative_impls)]
// Start of code
#![deny(clippy::correctness, clippy::complexity, clippy::all)]
#![warn(clippy::perf, clippy::suspicious, clippy::style)]
#![allow(dead_code)]
pub mod constants;
pub mod error;
pub mod instance;
pub mod library;
pub mod memory;
pub mod raw;
pub mod types;