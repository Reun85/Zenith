//! Better `syn::Error` handling.
//! Implements a `error::CombinedError` to collect multiple errors
//! WHY:
//! If possible its better to show the user multiple errors with all parts of the input.
//! So, for example, if the macro has a choice between A or B inputs, it can show the error with
//! either input aswell as the error for using both. So the user can make an educated guess as to
//! which it should be using.

use syn::Error;

#[derive(Debug, Clone, Default)]
pub struct CombinedError(pub Option<Error>);
impl CombinedError {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn combine(&mut self, x: Error) {
        match &mut self.0 {
            Some(err) => err.combine(x),
            None => self.0 = Some(x),
        };
    }
    pub fn create_new_error<S: std::fmt::Display + ?Sized>(
        &mut self,
        span: proc_macro2::Span,
        msg: &S,
    ) {
        use super::span::SpanMessages;
        self.combine(span.to_error(msg));
    }
    pub fn finish(self) -> Result<(), Error> {
        self.0.map_or(Ok(()), Err)
    }
    pub fn attach_result<T>(&mut self, inp: Result<T, Error>) -> Option<T> {
        match inp {
            Ok(x) => Some(x),
            Err(x) => {
                self.combine(x);
                None
            }
        }
    }
}

pub fn call_site_err<T: std::fmt::Display>(msg: T) -> syn::Error {
    syn::Error::new(proc_macro2::Span::call_site(), msg)
}

impl CombinedError {}

impl From<syn::Error> for CombinedError {
    fn from(val: Error) -> Self {
        Self(Some(val))
    }
}

impl From<CombinedError> for Result<(), syn::Error> {
    fn from(val: CombinedError) -> Self {
        val.finish()
    }
}

impl From<CombinedError> for Option<Error> {
    fn from(val: CombinedError) -> Self {
        val.0
    }
}

impl std::ops::DerefMut for CombinedError {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for CombinedError {
    type Target = Option<Error>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
