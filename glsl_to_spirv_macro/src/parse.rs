use super::ShaderSourceType;
use crate::utils::*;
use syn::spanned::Spanned;
use syn::*;

// pub const MANIFEST_DIR_ENV_VAR: &'static str = "CARGO_RUSTC_CURRENT_DIR";
pub const MANIFEST_DIR_ENV_VAR: &'static str = "CARGO_MANIFEST_DIR";
#[derive(Clone)]
pub(crate) struct MultiShaderInfo {
    pub(crate) name: Ident,
    pub(crate) data: ShaderSourceType,
    pub(crate) ty: Sp<shaderc::ShaderKind>,
    pub(crate) entry_point: Option<Ident>,
    pub(crate) generate_bindings: Option<LitBool>,
    pub(crate) generate_structure: Option<LitBool>,
}
#[derive(Clone)]
pub(crate) struct SingleShaderInfo {
    pub(crate) data: ShaderSourceType,
    pub(crate) ty: Sp<shaderc::ShaderKind>,
}
#[derive(Clone)]
pub(crate) enum InputType {
    Single(SingleShaderInfo),
    Multi(Vec<MultiShaderInfo>),
}
#[derive(Clone)]
pub(crate) struct Input {
    pub(crate) root: Sp<std::path::PathBuf>,
    pub(crate) shaders: InputType,
    pub(crate) generate_bindings: Option<LitBool>,
    pub(crate) generate_structure: Option<LitBool>,
    pub(crate) entry_point: Option<Ident>,
    pub(crate) vulkan_version: Option<Sp<shaderc::EnvVersion>>,
    pub(crate) spirv_version: Option<Sp<shaderc::SpirvVersion>>,
}

#[derive(Default)]
struct ShaderBuilderOuter {
    data: Option<KeyValue<Ident, ShaderSourceType>>,
    ty: Option<KeyValue<Ident, LitStr>>,
}
struct ShaderBuilder {
    name: Option<KeyValue<Ident, Ident>>,
    data: Option<KeyValue<Ident, ShaderSourceType>>,
    ty: Option<KeyValue<Ident, LitStr>>,
    entry_point: Option<KeyValue<Ident, LitStr>>,
    generate_bindings: Option<KeyValue<Ident, LitBool>>,
    generate_structure: Option<KeyValue<Ident, LitBool>>,
    missing_field_span: proc_macro2::Span,
}

fn parse_key_value_separator(input: &syn::parse::ParseStream) -> Result<()> {
    match input.parse::<Token![=]>() {
        Ok(_) => Ok(()),
        Err(_) => input.parse::<Token![:]>().map(|_| ()),
    }
}

impl Default for ShaderBuilder {
    fn default() -> Self {
        ShaderBuilder {
            name: None,
            data: None,
            ty: None,
            entry_point: None,
            generate_bindings: None,
            generate_structure: None,
            missing_field_span: proc_macro2::Span::call_site(),
        }
    }
}

fn check_for_duplicate_set<K: Spanned, V>(
    prev: &mut Option<KeyValue<K, V>>,
    new: KeyValue<K, V>,
    name: &str,
) {
    if let Some(previnner) = prev {
        new.key
            .span()
            .emit_warning(format!("`{}` has been set twice.", name));
        previnner
            .key
            .span()
            .emit_help(format!("`{}` previously set here", name));
    }
    *prev = Some(new);
}
impl syn::parse::Parse for ShaderBuilder {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut res = ShaderBuilder::default();
        while !input.is_empty() {
            let entry = input.parse::<Ident>()?;
            parse_key_value_separator(&input)?;

            match entry.to_string().as_str() {
                "ty" => {
                    let ty = input.parse::<LitStr>()?;
                    check_for_duplicate_set(&mut res.ty, KeyValue::new(entry, ty), "ty");
                }
                "name" => {
                    let name: Result<Ident> = match input.parse::<Ident>() {
                        Ok(x) => Ok(x),
                        // If its an err try to parse as string, then back to ident.
                        Err(e) => match input.parse::<LitStr>() {
                            Ok(x) => match syn::parse_str::<Ident>(&x.value()) {
                                Ok(y) => Ok(Ident::new(&y.to_string(), x.value().span())),
                                Err(_) => {
                                    return Err(e);
                                }
                            },
                            Err(_) => {
                                return Err(e);
                            }
                        },
                    };
                    let name = name?;
                    check_for_duplicate_set(&mut res.name, KeyValue::new(entry, name), "name")
                }
                "path" => {
                    let src = input.parse::<LitStr>()?;
                    check_for_duplicate_set(
                        &mut res.data,
                        KeyValue::new(entry, ShaderSourceType::Path(src)),
                        "shader source",
                    );
                }
                "data" | "src" | "source" => {
                    let data_syn = input.parse::<LitStr>()?;
                    check_for_duplicate_set(
                        &mut res.data,
                        KeyValue::new(entry, ShaderSourceType::Bytes(data_syn)),
                        "shader source",
                    );
                }
                "entry" | "entry_point" | "main" => {
                    let inp = input.parse::<LitStr>()?;
                    check_for_duplicate_set(
                        &mut res.entry_point,
                        KeyValue::new(entry, inp),
                        "entry_point",
                    );
                }
                "bind" | "bindings" | "gen_bindings" | "generate_bindings" => {
                    #[allow(unused_variables)]
                    let value = input.parse::<LitBool>()?;
                    check_for_duplicate_set(
                        &mut res.generate_bindings,
                        KeyValue::new(entry, value),
                        "generate_bindings",
                    );
                }
                "structure" | "gen_structure" | "generate_structure" => {
                    #[allow(unused_variables)]
                    let value = input.parse::<LitBool>()?;
                    check_for_duplicate_set(
                        &mut res.generate_structure,
                        KeyValue::new(entry, value),
                        "generate_structure",
                    );
                }
                _ => {
                    Err(syn::Error::new(entry.span(), "expected one of 'shaders`,`ty`,`generate_structure`,`generate_bindings`, `entry_point`, `src`|`path` keywords" ))?;
                }
            }
            // If its the end of the input, dont parse a comma.
            let comma = input.parse::<Token![,]>();
            if !input.is_empty() {
                comma?;
            }
        }
        res.missing_field_span = input.span();
        Ok(res)
    }
}
impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut shaders: Option<KeyValue<Ident, Vec<ShaderBuilder>>> = None;
        let mut outer = ShaderBuilderOuter::default();

        let mut entry_point: Option<KeyValue<Ident, LitStr>> = None;
        let mut vulkan_version: Option<KeyValue<Ident, LitStr>> = None;
        let mut spirv_version: Option<KeyValue<Ident, LitStr>> = None;

        let mut root: Option<KeyValue<Ident, LitStr>> = None;
        let mut generate_structure: Option<KeyValue<Ident, LitBool>> = None;
        let mut generate_bindings: Option<KeyValue<Ident, LitBool>> = None;

        let mut err = CombinedError::new();
        // Set a variable, and if it's already set, send a note.
        while !input.is_empty() {
            let entry = input.parse::<Ident>()?;
            parse_key_value_separator(&input)?;

            match entry.to_string().as_str() {
                "root" => {
                    let root_syn = input.parse::<LitStr>()?;
                    check_for_duplicate_set(&mut root, KeyValue::new(entry, root_syn), "root");
                }
                "ty" => {
                    let ty = input.parse::<LitStr>()?;
                    check_for_duplicate_set(&mut outer.ty, KeyValue::new(entry, ty), "ty");
                }
                "entry" | "entry_point" | "main" => {
                    let inp = input.parse::<LitStr>()?;
                    check_for_duplicate_set(
                        &mut entry_point,
                        KeyValue::new(entry, inp),
                        "entry_point",
                    );
                }
                "spirv" | "spirv_version" => {
                    let inp = input.parse::<LitStr>()?;
                    check_for_duplicate_set(
                        &mut spirv_version,
                        KeyValue::new(entry, inp),
                        "spirv_version",
                    );
                }
                "vulkan" | "vulkan_version" => {
                    let inp = input.parse::<LitStr>()?;
                    check_for_duplicate_set(
                        &mut vulkan_version,
                        KeyValue::new(entry, inp),
                        "vulkan_version",
                    );
                }
                "name" => {
                    let _ = input.parse::<LitStr>()?;
                    entry
                        .span()
                        .emit_warning("do not set `name` if specifying one shader");
                }
                "path" => {
                    let src_syn = input.parse::<LitStr>()?;
                    check_for_duplicate_set(
                        &mut outer.data,
                        KeyValue::new(entry, ShaderSourceType::Path(src_syn)),
                        "shader source",
                    );
                }
                "data" | "src" | "source" => {
                    let data_syn = input.parse::<LitStr>()?;
                    check_for_duplicate_set(
                        &mut outer.data,
                        KeyValue::new(entry, ShaderSourceType::Bytes(data_syn)),
                        "shader source",
                    );
                }
                "bind" | "bindings" | "gen_bindings" | "generate_bindings" => {
                    #[allow(unused_variables)]
                    let value = input.parse::<LitBool>()?;
                    check_for_duplicate_set(
                        &mut generate_bindings,
                        KeyValue::new(entry, value),
                        "generate_bindings",
                    );
                }
                "structure" | "gen_structure" | "generate_structure" => {
                    #[allow(unused_variables)]
                    let value = input.parse::<LitBool>()?;
                    check_for_duplicate_set(
                        &mut generate_structure,
                        KeyValue::new(entry, value),
                        "generate_structure",
                    );
                }
                "shaders" => {
                    let bracketed;
                    bracketed!(bracketed in input);
                    let mut shaders_builder: Vec<ShaderBuilder> = vec![];
                    while !bracketed.is_empty() {
                        let inner_brace;
                        braced!(inner_brace in bracketed);

                        shaders_builder.push(ShaderBuilder::parse(&inner_brace)?);
                        let comma = bracketed.parse::<Token![,]>();
                        if !bracketed.is_empty() {
                            comma?;
                        }
                    }
                    if shaders_builder.is_empty() {
                        let span = bracketed.span();
                        err.create_new_error(span, "expected at least one shader");
                    } else {
                        shaders = Some(KeyValue::new(entry, shaders_builder));
                    }
                }
                _ => {
                    Err(syn::Error::new(entry.span(), "expected one of 'shaders`,`ty`,`generate_structure`,`generate_bindings`,`entry_point`,`vulkan_version`,`spirv_version`,`,src`|`data` keywords" ))?;
                }
            }
            // If its the end of the input, dont parse a comma.
            let comma = input.parse::<Token![,]>();
            if !input.is_empty() {
                comma?;
            }
        }
        // All variables are set, check content for conflicts.

        let generate_bindings = generate_bindings.map(|x| x.value);
        let generate_structure = generate_structure.map(|x| x.value);
        let entry_point = entry_point.and_then(|x| match syn::parse_str::<Ident>(&x.value()) {
            Ok(_) => Some(Ident::new(&x.value(), x.value().span())),
            Err(e) => {
                err.create_new_error(x.value().span(), e.to_string().as_str());
                None
            }
        });
        let vulkan_version = vulkan_version.and_then(|x| match x.value().as_str() {
            "1.0" | "1_0" | "v1.0" | "v1_0" => {
                Some(Sp::new(shaderc::EnvVersion::Vulkan1_0, x.value().span()))
            }
            "1.1" | "1_1" | "v1.1" | "v1_1" => {
                Some(Sp::new(shaderc::EnvVersion::Vulkan1_1, x.value().span()))
            }
            "1.2" | "1_2" | "v1.2" | "v1_2" => {
                Some(Sp::new(shaderc::EnvVersion::Vulkan1_2, x.value().span()))
            }
            "1.3" | "1_3" | "v1.3" | "v1_3" => {
                Some(Sp::new(shaderc::EnvVersion::Vulkan1_3, x.value().span()))
            }
            _ => {
                err.create_new_error(x.value.span(), "expected one of `1.0`, `1.1`, `1.2`, `1.3`");
                None
            }
        });

        let spirv_version = spirv_version.and_then(|x| match x.value().as_str() {
            "1.0" | "1_0" | "v1.0" | "v1_0" => {
                Some(Sp::new(shaderc::SpirvVersion::V1_0, x.value().span()))
            }
            "1.1" | "1_1" | "v1.1" | "v1_1" => {
                Some(Sp::new(shaderc::SpirvVersion::V1_1, x.value().span()))
            }
            "1.2" | "1_2" | "v1.2" | "v1_2" => {
                Some(Sp::new(shaderc::SpirvVersion::V1_2, x.value().span()))
            }
            "1.3" | "1_3" | "v1.3" | "v1_3" => {
                Some(Sp::new(shaderc::SpirvVersion::V1_3, x.value().span()))
            }
            "1.4" | "1_4" | "v1.4" | "v1_4" => {
                Some(Sp::new(shaderc::SpirvVersion::V1_4, x.value().span()))
            }
            "1.5" | "1_5" | "v1.5" | "v1_5" => {
                Some(Sp::new(shaderc::SpirvVersion::V1_5, x.value().span()))
            }
            "1.6" | "1_6" | "v1.6" | "v1_6" => {
                Some(Sp::new(shaderc::SpirvVersion::V1_6, x.value().span()))
            }
            _ => {
                err.create_new_error(
                    x.value().span(),
                    "expected one of `1.0`, `1.1`, `1.2`, `1.3`, `1.4`, `1.5`, `1.6`",
                );
                None
            }
        });

        let shaders = if let Some(shaders) = shaders {
            let conflict = {
                let mut conflict = false;
                if let Some(ty) = &outer.ty {
                    err.create_new_error(ty.key.span(), "do not set `ty` if `shaders` is set.");
                    conflict = true;
                }
                if let Some(data) = &outer.data {
                    err.create_new_error(
                        data.key.span(),
                        format!("do not set `{}` if `shaders` is set.", data.key).as_str(),
                    );
                    conflict = true;
                }

                conflict
            };
            if conflict {
                shaders.key.span().emit_help("`shaders` set here");
            }
            let inner = err.attach_result(validate_multi(shaders.value));
            err.finish()?;
            InputType::Multi(inner.unwrap())
        } else {
            let inner = err.attach_result(validate_single(outer));
            err.finish()?;
            InputType::Single(inner.unwrap())
        };

        let root = match root {
            Some(x) => {
                let given = std::path::PathBuf::from(x.value());
                match given.is_absolute() {
                    true => Sp::new(given, x.value.span()),
                    false => Sp::new(
                        std::path::PathBuf::from(
                            std::env::var(MANIFEST_DIR_ENV_VAR).unwrap_or_else(|_| ".".into()),
                        )
                        .join(given),
                        x.value.span(),
                    ),
                }
            }
            None => Sp::new_call_site(std::path::PathBuf::from(
                std::env::var(MANIFEST_DIR_ENV_VAR).unwrap_or_else(|_| ".".into()),
            )),
        };

        Ok(Self {
            root,
            shaders,
            generate_bindings,
            generate_structure,
            entry_point,
            spirv_version,
            vulkan_version,
        })
    }
}

fn validate_single(inp: ShaderBuilderOuter) -> Result<SingleShaderInfo> {
    let missing_field_span = proc_macro2::Span::call_site();
    let mut err = CombinedError::new();
    let ty;
    let fine = {
        let mut fine = true;
        if inp.data.is_none() {
            err.create_new_error(missing_field_span, "missing field 'src` or `path`");
            fine = false;
        }
        ty = match inp.ty {
            None => {
                err.create_new_error(missing_field_span, "missing field 'ty`");

                fine = false;
                None
            }
            Some(KeyValue {
                key: _,
                value: inner,
            }) => match parse_shader_ty(inner.value().as_str()) {
                Ok(x) => Some(Sp::new(x, inner.value().span())),
                Err(e) => {
                    err.create_new_error(inner.span(), e);
                    fine = false;
                    None
                }
            },
        };
        fine
    };
    if fine {
        Ok(SingleShaderInfo {
            data: inp.data.unwrap().value,
            ty: ty.unwrap(),
        })
    } else {
        Err(err.0.unwrap())
    }
}
fn validate_multi(inp: Vec<ShaderBuilder>) -> Result<Vec<MultiShaderInfo>> {
    let mut err = CombinedError::new();
    let mut duplicates = vec![false; inp.len()];
    for (i, shader) in inp.iter().enumerate() {
        if duplicates[i] {
            continue;
        }

        if let Some(KeyValue {
            key: _,
            value: name,
        }) = &shader.name
        {
            let outername = name.to_string();
            for x in (i + 1)..inp.len() {
                if duplicates[x] {
                    continue;
                }
                if let Some(KeyValue {
                    key: _,
                    value: name2,
                }) = &inp[x].name
                {
                    duplicates[x] = outername == name2.to_string();
                }
            }
        }
    }
    for (ind, val) in duplicates.iter().enumerate() {
        if *val {
            let shader = &inp[ind];
            if let Some(KeyValue { key: _, value: n }) = &shader.name {
                err.create_new_error(n.span(), "shaders name's have to be unique");
            }
        }
    }
    let res = inp
        .into_iter()
        .filter_map(|res| {
            let ty;
            let name;
            let entry_point;
            let fine = {
                let mut fine = true;
                if res.data.is_none() {
                    err.create_new_error(res.missing_field_span, "missing field 'src` or `path`");
                    fine = false;
                }
                name = if let Some(KeyValue { key: _, value: x }) = res.name {
                    Some(x)
                } else {
                    err.create_new_error(res.missing_field_span, "missing field 'name`");
                    fine = false;
                    None
                };
                entry_point = if let Some(KeyValue { key: _, value: x }) = res.entry_point {
                    match syn::parse_str::<Ident>(&x.value()) {
                        Ok(_) => Some(Ident::new(&x.value(), x.value().span())),
                        Err(e) => {
                            err.create_new_error(x.value().span(), e.to_string().as_str());
                            fine = false;
                            None
                        }
                    }
                } else {
                    None
                };
                ty = match res.ty {
                    None => {
                        err.create_new_error(res.missing_field_span, "missing field 'ty`");

                        fine = false;
                        None
                    }
                    Some(KeyValue {
                        key: _,
                        value: inner,
                    }) => match parse_shader_ty(inner.value().as_str()) {
                        Ok(x) => Some(Sp::new(x, inner.value().span())),
                        Err(e) => {
                            err.create_new_error(inner.span(), e);
                            fine = false;
                            None
                        }
                    },
                };
                fine
            };
            if fine {
                Some(MultiShaderInfo {
                    name: name.unwrap(),
                    data: res.data.unwrap().value,
                    ty: ty.unwrap(),
                    entry_point,
                    generate_bindings: res.generate_bindings.map(|x| x.value),
                    generate_structure: res.generate_structure.map(|x| x.value),
                })
            } else {
                None
            }
        })
        .collect();
    err.finish()?;
    Ok(res)
}

fn parse_shader_ty(inp: &str) -> std::result::Result<shaderc::ShaderKind, &'static str> {
    match inp {
        "vertex" | "vert" => Ok(shaderc::ShaderKind::Vertex),
        "fragment" | "frag" => Ok(shaderc::ShaderKind::Fragment),
        "compute" | "comp" => Ok(shaderc::ShaderKind::Compute),
        "geometry" | "geo" | "geom" => Ok(shaderc::ShaderKind::Geometry),
        "tese" | "tesseval" => Ok(shaderc::ShaderKind::TessEvaluation),
        "tesc" | "tesscontrol" => Ok(shaderc::ShaderKind::TessControl),
        "mesh" => Ok(shaderc::ShaderKind::Mesh),
        "task" => Ok(shaderc::ShaderKind::Task),
        "rgen" => Ok(shaderc::ShaderKind::RayGeneration),
        "rint" => Ok(shaderc::ShaderKind::Intersection),
        "rahit" => Ok(shaderc::ShaderKind::AnyHit),
        "rchit" => Ok(shaderc::ShaderKind::ClosestHit),
        "rmiss" => Ok(shaderc::ShaderKind::Miss),
        "rcall" => Ok(shaderc::ShaderKind::Callable),

        _ => Err("expected one of `vertex`, `fragment`, `compute`, `geometry`, `tesseval`,`tesscontrol`, `mesh`, `task`, `rgen`, `rint`, `rahit`, `rchit`, `rmiss`, `rcall`"),
    }
}
