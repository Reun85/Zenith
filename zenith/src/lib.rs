#![deny(clippy::correctness, clippy::complexity, clippy::all)]
#![warn(
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
    clippy::suspicious,
    clippy::style
)]

pub mod log;
pub mod shaders;
pub mod undrop;

fn main() -> Result<(), ()> {
    Ok(())
}
