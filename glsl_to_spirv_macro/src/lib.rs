#![feature(proc_macro_span)]
#![feature(track_path)]
#![deny(clippy::correctness, clippy::complexity, clippy::all)]
#![warn(
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
    clippy::suspicious,
    clippy::style
)]

extern crate spirv_reflect;
mod compiler;
mod parse;
mod shadercserde;
mod tokengeneration;
mod utils;
use std::borrow::Cow;

use crate::parse::{Input, InputType};
use syn::{spanned::Spanned, Ident, LitStr};
use utils::{CombinedError, Sp, SpanMessages};

// #[derive(Debug, Clone)]
// #[non_exhaustive]
// enum ShaderTypes {
//     Vertex,
//     Fragment,
//     Compute,
//     Geometry,
//     Tesselation,
// }

#[derive(Clone)]
enum ShaderSourceType {
    Bytes(LitStr),
    Path(LitStr),
}
impl ShaderSourceType {
    fn span(&self) -> proc_macro2::Span {
        match self {
            Self::Path(x) | Self::Bytes(x) => x.span(),
        }
    }
}

#[derive(Clone)]
struct ShaderInfo<'a> {
    root: &'a Sp<std::path::PathBuf>,
    // If its a single shader, this is None
    name: Option<Ident>,
    data: ShaderSourceType,
    ty: Sp<shaderc::ShaderKind>,
    entry_point: Cow<'a, Ident>,
    vulkan_version: Cow<'a, Sp<shaderc::EnvVersion>>,
    spirv_version: Cow<'a, Sp<shaderc::SpirvVersion>>,
    generate_structure: Cow<'a, Sp<bool>>,
    generate_bindings: Cow<'a, Sp<bool>>,
}

#[proc_macro]
pub fn shader(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let time = std::time::Instant::now();
    let input = match syn::parse::<Input>(input) {
        Ok(x) => x,
        Err(e) => return e.to_compile_error().into(),
    };
    let ret = shader_inner(input).unwrap_or_else(syn::Error::into_compile_error);
    let end = std::time::Instant::now();
    ret.span().emit_note(format!("{:?}", end - time));
    ret.into()
}

fn shader_inner(input: Input) -> std::result::Result<proc_macro2::TokenStream, syn::Error> {
    let root = input.root;
    let default_vulkan_version = input
        .vulkan_version
        .unwrap_or_else(|| Sp::new_call_site(shaderc::EnvVersion::Vulkan1_3));
    let default_spirv_version: Sp<shaderc::SpirvVersion> = input
        .spirv_version
        .unwrap_or_else(|| Sp::new_call_site(shaderc::SpirvVersion::V1_6));
    let default_generate_structure = input
        .generate_structure
        .map_or_else(|| Sp::new_call_site(true), |x| Sp::new(x.value(), x.span()));
    let default_generate_bindings = input
        .generate_bindings
        .map_or_else(|| Sp::new_call_site(true), |x| Sp::new(x.value(), x.span()));
    let default_entry_point = input
        .entry_point
        .unwrap_or_else(|| Ident::new("main", proc_macro2::Span::call_site()));

    let compiled_value = {
        let compiler = compiler::Compiler::init().map_err(|x| match x {
            compiler::InitError::CompilerMissing => {
                syn::Error::new(proc_macro2::Span::call_site(), x.to_string())
            }
        })?;

        match input.shaders {
            InputType::Single(shader) => {
                let info = ShaderInfo {
                    root: &root,
                    name: None,
                    data: shader.data,
                    ty: shader.ty,
                    entry_point: Cow::Owned(default_entry_point),
                    vulkan_version: Cow::Owned(default_vulkan_version),
                    spirv_version: Cow::Owned(default_spirv_version),
                    generate_structure: Cow::Owned(default_generate_structure),
                    generate_bindings: Cow::Owned(default_generate_bindings),
                };
                let (words, reflec) = compiler.get_words_and_reflection(&info)?;
                vec![(info, words, reflec)].into_iter()
            }

            InputType::Multi(shaders) => {
                let mut err = CombinedError::new();
                let compiled_values = shaders
                    .into_iter()
                    .map(|shader| ShaderInfo {
                        root: &root,
                        name: Some(shader.name),
                        data: shader.data,
                        ty: shader.ty,
                        entry_point: shader
                            .entry_point
                            .map_or_else(|| Cow::Borrowed(&default_entry_point), Cow::Owned),
                        spirv_version: Cow::Borrowed(&default_spirv_version),
                        vulkan_version: Cow::Borrowed(&default_vulkan_version),
                        generate_structure: shader.generate_structure.map_or_else(
                            || Cow::Borrowed(&default_generate_structure),
                            |x| Cow::Owned(Sp::new(x.value(), x.span())),
                        ),
                        generate_bindings: shader.generate_bindings.map_or_else(
                            || Cow::Borrowed(&default_generate_bindings),
                            |x| Cow::Owned(Sp::new(x.value(), x.span())),
                        ),
                    })
                    .filter_map(|x| match compiler.get_words_and_reflection(&x) {
                        Ok((words, reflec)) => Some((x, words, reflec)),
                        Err(e) => {
                            err.combine(e);
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .into_iter();
                err.finish()?;

                compiled_values
            }
        }
    };

    // We have successfully compiled all the shaders at this point. Is storing all of them in a
    // vec resource intensive? Doubt.
    let iter = compiled_value
        .map(|(info, words, reflec)| tokengeneration::to_tokens(info, words.words(), &reflec))
        .collect::<syn::Result<Vec<_>>>()?;
    let tokens = quote::quote!(#(#iter)*);
    Ok(tokens)
}
