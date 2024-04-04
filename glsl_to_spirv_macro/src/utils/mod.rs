#![allow(dead_code)]
mod error;
mod key_value;
mod span;

pub use error::*;
pub use key_value::*;
pub use span::*;

pub(crate) fn debug_msg<T: std::fmt::Display>(msg: T) {
    if cfg!(debug_assertions) {
        proc_macro2::Span::call_site().emit_note(msg);
    }
}
