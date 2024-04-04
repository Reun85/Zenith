use core::fmt;
use proc_macro2_diagnostics::SpanDiagnosticExt;
use quote::ToTokens;
use std::ops::{Deref, DerefMut};

pub trait SpanMessages {
    fn to_error(self, msg: impl fmt::Display) -> syn::Error;
    fn emit_error(self, msg: impl fmt::Display);
    fn emit_note(self, msg: impl fmt::Display);
    fn emit_help(self, msg: impl fmt::Display);
    fn emit_warning(self, msg: impl fmt::Display);
}
impl SpanMessages for proc_macro2::Span {
    fn to_error(self, msg: impl fmt::Display) -> syn::Error {
        syn::Error::new(self, msg)
    }
    fn emit_error(self, msg: impl fmt::Display) {
        self.error(msg.to_string()).emit_as_item_tokens();
    }
    fn emit_note(self, msg: impl fmt::Display) {
        self.note(msg.to_string()).emit_as_item_tokens();
    }
    fn emit_help(self, msg: impl fmt::Display) {
        self.help(msg.to_string()).emit_as_item_tokens();
    }
    fn emit_warning(self, msg: impl fmt::Display) {
        self.warning(msg.to_string()).emit_as_item_tokens();
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Sp<T> {
    pub(crate) value: T,
    pub(crate) span: proc_macro2::Span,
}
impl<T> Sp<T> {
    pub fn new(value: T, span: proc_macro2::Span) -> Self {
        Self { value, span }
    }
    pub fn new_call_site(value: T) -> Self {
        Self::new(value, proc_macro2::Span::call_site())
    }
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn span(&self) -> &proc_macro2::Span {
        &self.span
    }
}

impl<T> DerefMut for Sp<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
impl<T> Deref for Sp<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl From<syn::LitStr> for Sp<String> {
    fn from(value: syn::LitStr) -> Self {
        Self::new(value.value(), value.span())
    }
}
impl From<syn::LitBool> for Sp<bool> {
    fn from(value: syn::LitBool) -> Self {
        Self::new(value.value, value.span())
    }
}

impl<T: ToTokens> ToTokens for Sp<T> {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let tt = self.value.to_token_stream().into_iter().map(|mut tt| {
            tt.set_span(self.span);
            tt
        });

        stream.extend(tt);
    }
}
