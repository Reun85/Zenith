use crate::utils::*;
pub(crate) fn to_tokens(
    info: crate::ShaderInfo<'_>,
    words: Vec<u32>,
    reflection: spirv_reflect::Reflection,
) -> syn::Result<proc_macro2::TokenStream> {
    let visibility = quote::quote! {pub};
    let span = info.data.span();
    let load_words_doc = "The content of the SPIR-V compiled shader as a slice of u32 'words'.";
    let load_func = {
        let len = words.len();
        quote::quote_spanned! {span=>
                #[must_use]
                #[doc = #load_words_doc]
            #visibility fn load_words() -> &'static [u32] {
                const WORDS: [u32; #len] = [#(#words),*];
                &WORDS
            }
        }
    };
    let data = {
        // spirv_reflect uses spirv_tools :)

        if *info.generate_bindings.value() {
            create_data_rep(&reflection).map_err(|x| span.to_error(x.to_string()))?
        } else {
            proc_macro2::TokenStream::new()
        }
    };
    Ok(match info.name {
        None => {
            quote::quote_spanned!(span=>
                #load_func
                #data
            )
        }
        Some(name) => {
            quote::quote_spanned! { span=>
                        mod #name {
                           #load_func
                           #data
                        }
            }
        }
    })
}

fn create_data_rep(
    reflec: &spirv_reflect::Reflection,
) -> Result<proc_macro2::TokenStream, spirv_reflect::ReflectError> {
    let debug_names = reflec.get_debug_names()?;
    let input_variables =
        reflec.get_all_variables_with_storage_class(spirv_reflect::spirv::StorageClass::Input)?;
    let uniform_variables =
        reflec.get_all_variables_with_storage_class(spirv_reflect::spirv::StorageClass::Uniform)?;
    let constant_variables = reflec.get_all_variables_with_storage_class(
        spirv_reflect::spirv::StorageClass::UniformConstant,
    )?;
    let output_variables =
        reflec.get_all_variables_with_storage_class(spirv_reflect::spirv::StorageClass::Output)?;
    let decoration = reflec.get_decorations();
    let member_decoration = reflec.get_member_decoration();
    let types = reflec.get_types()?;
    let inputs = input_variables
        .iter()
        .map(|x| {
            let name = debug_names.get_name(*x).unwrap();
            syn::Ident::new(&name, proc_macro2::Span::call_site())
        })
        .collect::<Vec<_>>();
    let inputstype = input_variables
        .iter()
        .map(|x| {
            let ty = types
                .get(&reflec.get_type_of_variable(*x).unwrap())
                .unwrap();
            ty.to_tokens()
        })
        .collect::<Vec<_>>();
    let input_struct_docs = "The input structure for the shader through the pipeline";
    let input_struct = quote::quote! {
        #[repr(C)]
        #[derive(Debug, Clone, Copy)]
        #[doc = #input_struct_docs]
        #[automatically_derived]
        pub struct Input {
            #(
                pub #inputs: #inputstype,
            )*
        }
    };

    Ok(quote::quote! {
        #input_struct
    })
}

trait ToTokens {
    fn to_tokens(&self) -> proc_macro2::TokenStream;
}

impl ToTokens for spirv_reflect::types::Int {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let ty = if self.issigned { "i" } else { "u" };
        let full = format!("{}{}", ty, self.bits);
        let full: syn::Type = syn::parse_str(&full).unwrap();
        quote::quote! (#full)
    }
}
impl ToTokens for spirv_reflect::types::Float {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let ty = "f";
        let full = format!("{}{}", ty, self.bits);
        let full: syn::Type = syn::parse_str(&full).unwrap();
        quote::quote! (#full)
    }
}

impl ToTokens for spirv_reflect::types::Vector {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let len: u16 = self.size.into();
        let full = format!("Vector{}", len);
        let full: syn::Type = syn::parse_str(&full).unwrap();
        let inner = self.inner_type.to_tokens();
        quote::quote! (nalgebra::#full<#inner>)
    }
}
impl ToTokens for spirv_reflect::types::Mat {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        debug_assert!(matches!(
            *self.inner_type,
            spirv_reflect::types::Type::Vector(spirv_reflect::types::Vector {
                inner_type:_,
                size,
            })
        if size==self.size));
        let len: u16 = self.size.into();
        let full = format!("Matrix{}", len);
        let full: syn::Type = syn::parse_str(&full).unwrap();
        let inner = self.inner_type.to_tokens();
        quote::quote! (nalgebra::#full<#inner>)
    }
}
impl ToTokens for spirv_reflect::types::Array {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let inner = self.inner_type.to_tokens();
        let len = self.len;
        quote::quote! ([#inner; #len])
    }
}

impl ToTokens for spirv_reflect::types::RunTimeArray {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let inner = self.inner_type.to_tokens();
        quote::quote! (Vec<#inner>)
    }
}

impl ToTokens for spirv_reflect::types::Type {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        match self {
            spirv_reflect::types::Type::Int(x) => x.to_tokens(),
            spirv_reflect::types::Type::Float(x) => x.to_tokens(),
            spirv_reflect::types::Type::Vector(x) => x.to_tokens(),
            spirv_reflect::types::Type::Mat(x) => x.to_tokens(),
            spirv_reflect::types::Type::Struct(_) => todo!(),
            spirv_reflect::types::Type::Array(x) => x.to_tokens(),
            spirv_reflect::types::Type::RunTimeArray(x) => x.to_tokens(),
        }
    }
}
