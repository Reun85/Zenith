use std::{borrow::Cow, cell::RefCell, os::unix::fs::MetadataExt, rc::Rc};

use crate::utils::*;

pub struct Compiler {
    shader_compiler: shaderc::Compiler,
}

#[derive(Debug)]
#[non_exhaustive]
#[derive(thiserror::Error)]
pub enum CompilerInitError {
    #[error("Could not get shaderc compiler")]
    CompilerMissing,
}
#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct CompileArtifacts {
    words: Vec<u32>,
    includes: Vec<std::path::PathBuf>,
}
impl CompileArtifacts {
    pub fn words(&self) -> &[u32] {
        &self.words
    }
    pub fn includes(&self) -> &[std::path::PathBuf] {
        &self.includes
    }
}

impl Compiler {
    pub fn init() -> Result<Self, CompilerInitError> {
        let ok_or = shaderc::Compiler::new().ok_or(CompilerInitError::CompilerMissing);
        Ok(Self {
            shader_compiler: ok_or?,
        })
    }
    pub fn compile(&self, input: &crate::ShaderInfo) -> syn::Result<CompileArtifacts> {
        let input: &crate::ShaderInfo = &input;
        let shader_kind = input.ty.value();

        let (source_text, name_for_errors) = match input.data {
            crate::ShaderSourceType::Bytes(ref bytes) => (
                bytes.value(),
                input
                    .name
                    .as_ref()
                    .map(|x| x.to_string())
                    .unwrap_or("shader".to_string()),
            ),

            crate::ShaderSourceType::Path(ref file) => {
                let mut path = input.root.value().clone();
                path.push(file.value());
                if !path.exists() {
                    return Err(file
                        .span()
                        .to_error(format!("file does not exist {:?}", path)));
                }
                proc_macro::tracked_path::path(path.to_str().unwrap());
                let res = std::fs::read_to_string(path)
                    .map_err(|_| file.span().to_error("could not read source file."))?;
                (res, file.value())
            }
        };
        let entry_point_name = input.entry_point.to_string();
        let mut additional_options = shaderc::CompileOptions::new()
            .ok_or(call_site_err("could not create shaderc::CompileOptions"))?;
        additional_options.set_target_env(
            shaderc::TargetEnv::Vulkan,
            input.vulkan_version.value as u32,
        );
        additional_options.set_target_spirv(input.spirv_version.value);
        additional_options.set_auto_bind_uniforms(true);
        additional_options.set_auto_map_locations(true);
        additional_options.set_auto_combined_image_sampler(true);
        let include_folder = std::path::PathBuf::from(&input.root.value);
        let includes: Rc<RefCell<Vec<std::path::PathBuf>>> = Rc::new(RefCell::new(vec![]));
        let included_paths = includes.clone();
        additional_options.set_include_callback(move |inc, ty, name_of_file, _| {
            let mut path = match ty {
                shaderc::IncludeType::Relative => {
                    let path = std::path::PathBuf::from(&name_of_file);
                    path.parent()
                        .map(std::path::Path::to_path_buf)
                        .map(|x| include_folder.clone().join(x))
                        .unwrap_or(include_folder.clone())
                }
                shaderc::IncludeType::Standard => include_folder.clone(),
            };
            path.push(inc);

            if !path.exists() {
                return Err(format!("file does not exist {:?}", path));
            }
            let res = std::fs::read_to_string(path.clone()).map_err(|_| {
                format!(
                    "could not read source file {:?} for include {:?}",
                    path, inc
                )
            })?;
            included_paths.borrow_mut().push(path);
            Ok(shaderc::ResolvedInclude {
                resolved_name: inc.into(),
                content: res,
            })
        });
        #[cfg(feature = "generate_debug_info")]
        additional_options.set_generate_debug_info();
        let artifact = self
            .shader_compiler
            .compile_into_spirv(
                source_text.as_str(),
                *shader_kind,
                name_for_errors.as_str(),
                entry_point_name.as_str(),
                Some(&additional_options),
            )
            .map_err(|x| input.data.span().to_error(x.to_string()))?;
        if artifact.get_num_warnings() != 0 {
            input
                .data
                .span()
                .emit_warning(artifact.get_warning_messages());
        }
        let words = artifact.as_binary().into();
        // This should drop included_paths
        drop(artifact);
        drop(additional_options);
        // TODO: This copies this data.
        let includes: Vec<std::path::PathBuf> = includes.borrow().clone();
        Ok(CompileArtifacts { words, includes })
    }
    fn local_to_manifest(
        &self,
        root: &std::path::Path,
        path: &std::path::PathBuf,
    ) -> std::path::PathBuf {
        let manifest_dir = match std::env::var("CARGO_RUSTC_CURRENT_DIR") {
            Ok(x) => x,
            Err(_) => return path.clone(),
        };
        let manifest_dir = std::path::PathBuf::from(manifest_dir);
        let mut full: std::path::PathBuf = root.into();
        full.push(path);
        if full.starts_with(&manifest_dir) {
            full.strip_prefix(&manifest_dir).unwrap().to_path_buf()
        } else {
            path.clone()
        }
    }
    // This was a very noble idea, but sadly as long as the compiled glsl files can include external
    // files, this would become way to cumbersome to actually use.
    fn try_and_load_from_checkpoint(
        &self,
        info: &crate::ShaderInfo<'_>,
    ) -> syn::Result<(CompileArtifacts, spirv_reflect::Reflection)> {
        let span = info.data.span();
        // Get the metadata of the actual file compare that to the previous metadata.....
        // Load previous info
        const FILE_EXT: &str = ".bin";
        let out_dir = std::env!("OUT_DIR");

        let file_name = {
            let file_name = match &info.data {
                crate::ShaderSourceType::Bytes(_) => None,
                crate::ShaderSourceType::Path(path) => Some(std::path::PathBuf::from(path.value())),
            };
            let file_name = file_name
                .as_ref()
                .map(|x| self.local_to_manifest(&info.root.value(), x));
            let file_name = match file_name {
                Some(mut path) => {
                    let mut ext = path.extension().unwrap_or_default().to_os_string();
                    ext.push(FILE_EXT);
                    path.set_extension(ext);
                    Some(std::path::PathBuf::from(out_dir).join(path))
                }
                None => None,
            };
            file_name
        };

        let config_file = file_name.as_ref().and_then(|x| std::fs::File::open(x).ok());

        let previous_info = config_file.as_ref().and_then(|file| {
            let res: Option<InnerInfo> =
                bincode::deserialize_from(std::io::BufReader::new(file)).ok();
            // let res: Option<InnerInfo> = serde_json::de::from_reader(file).ok();
            res
        });
        // Check the source file
        //

        let file: Option<std::fs::File> = {
            match info.data {
                crate::ShaderSourceType::Bytes(_) => None,
                crate::ShaderSourceType::Path(ref file) => {
                    let mut path = info.root.value().clone();
                    path.push(file.value());
                    if !path.exists() {
                        None
                    } else {
                        std::fs::File::open(path).ok()
                    }
                }
            }
        };
        let time_changed = file.and_then(|file| file.metadata().ok().map(|x| x.mtime() as u128));

        let file_changed = time_changed.is_some_and(|x| {
            previous_info
                .as_ref()
                .is_some_and(|y| x != y.last_changed_time)
        });
        let need_to_save_new_info =
            file_name.is_some() && (info.needs_to_save_new_info(&previous_info) || file_changed);
        let should_recompile = info.need_to_recompile(&previous_info)
            || match previous_info {
                Some(ref x) => x.generated_binary.is_none(),
                None => true,
            }
            || file_changed;

        let mut recompiled = false;
        let (words, reflection) = if !should_recompile {
            let words = previous_info.and_then(|x| x.generated_binary);
            let reflection = words
                .as_ref()
                .and_then(|x| spirv_reflect::Reflection::new_from_spirv_words(&x.words).ok());
            (words, reflection)
        } else {
            (None, None)
        };

        let (words, reflection) = match (words, reflection) {
            (Some(words), Some(reflection)) => (
                match words {
                    Cow::Borrowed(x) => x.clone(),
                    Cow::Owned(x) => x,
                },
                reflection,
            ),
            _ => {
                recompiled = true;
                let iwords = self.compile(info)?;
                let ireflection = spirv_reflect::Reflection::new_from_spirv_words(&iwords.words)
                    .map_err(|err| span.to_error(err.to_string()))?;
                (iwords, ireflection)
            }
        };
        if need_to_save_new_info || recompiled {
            let inner_info = InnerInfo::from(
                info.clone(),
                time_changed.unwrap_or(0),
                Some(Cow::Borrowed(&words)),
            );
            // let res = ron::to_string(&inner_info);
            // let res = serde_json::to_string(&inner_info);
            match bincode::serialize(&inner_info) {
                Ok(res) => {
                    let file_name = file_name.unwrap();
                    std::fs::create_dir_all(file_name.parent().unwrap()).unwrap();
                    let x = std::fs::write(file_name, res);

                    if let Err(x) = x {
                        span.emit_warning(format!("Could not save new info {:?}", x));
                    }
                }
                Err(x) => {
                    span.emit_warning(format!("Could not save new info {:?}", x));
                }
            }
        }

        Ok((words, reflection))
    }
    pub(crate) fn get_words_and_reflection(
        &self,
        info: &crate::ShaderInfo<'_>,
    ) -> syn::Result<(CompileArtifacts, spirv_reflect::Reflection)> {
        self.try_and_load_from_checkpoint(info)
    }

    fn basic_compile_step(
        &self,
        info: &crate::ShaderInfo<'_>,
    ) -> std::result::Result<(Vec<u32>, spirv_reflect::Reflection), syn::Error> {
        let span = info.data.span();
        let compile_artifacts = self.compile(info)?;
        let ireflection = spirv_reflect::Reflection::new_from_spirv_words(&compile_artifacts.words)
            .map_err(|err| span.to_error(err.to_string()))?;
        Ok((compile_artifacts.words, ireflection))
    }
}

#[derive(PartialEq, Eq, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct InnerInfo<'a> {
    // If its a single shader, this is None
    #[serde(with = "crate::shadercserde::MyShaderKind")]
    ty: shaderc::ShaderKind,
    entry_point: String,
    #[serde(with = "crate::shadercserde::MyEnvVersion")]
    vulkan_version: shaderc::EnvVersion,
    #[serde(with = "crate::shadercserde::MySpirvVersion")]
    spirv_version: shaderc::SpirvVersion,
    generated_binary: Option<Cow<'a, CompileArtifacts>>,
    last_changed_time: u128,
}
impl<'a> InnerInfo<'a> {
    fn from(
        value: crate::ShaderInfo<'a>,
        last_changed_time: u128,
        generated_binary: Option<Cow<'a, CompileArtifacts>>,
    ) -> Self {
        Self {
            ty: *value.ty.value(),
            entry_point: value.entry_point.as_ref().to_string(),
            vulkan_version: *value.vulkan_version.as_ref().value(),
            spirv_version: *value.spirv_version.clone().value(),
            last_changed_time,
            generated_binary,
        }
    }
}
impl<'a> crate::ShaderInfo<'a> {
    pub fn need_to_recompile(&self, previous: &Option<InnerInfo>) -> bool {
        let previous = match previous {
            None => return true,
            Some(x) => x,
        };
        previous.ty != *self.ty
            || previous.entry_point != *self.entry_point.to_string()
            || previous.vulkan_version != **self.vulkan_version
            || previous.spirv_version != **self.spirv_version
            || matches!(self.data, crate::ShaderSourceType::Bytes(_))
    }
    pub fn needs_to_save_new_info(&self, previous: &Option<InnerInfo>) -> bool {
        let previous = match previous {
            None => return true,
            Some(x) => x,
        };

        *self.ty != previous.ty
            || self.entry_point.to_string() != previous.entry_point
            || **self.vulkan_version != previous.vulkan_version
            || **self.spirv_version != previous.spirv_version
    }
}
