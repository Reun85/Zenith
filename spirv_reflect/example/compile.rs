use std::path::PathBuf;
pub fn compile(path: PathBuf) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut resources = MacroResources::init()?;
    Ok(resources.compile_code(path).unwrap())
}

pub struct MacroResources {
    // Should be global.
    shader_compiler: shaderc::Compiler,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MacroInitError {
    #[error("could not create shaderc::Compiler")]
    ShadercCompiler,
}
impl MacroResources {
    pub fn init() -> Result<Self, MacroInitError> {
        Ok(Self {
            shader_compiler: shaderc::Compiler::new().ok_or(MacroInitError::ShadercCompiler)?,
        })
    }
    fn compile_code(&mut self, path: PathBuf) -> Result<Vec<u8>, String> {
        let shader_kind = shaderc::ShaderKind::Vertex;

        let (source_text, name_for_errors) = {
            let file = path.display().to_string();
            if !path.exists() {
                return Err("file does not exist {:?}".into());
            }
            let res = std::fs::read_to_string(path).map_err(|_| "could not read source file.")?;
            (res, file)
        };
        let entry_point_name = "main";
        let mut additional_options =
            shaderc::CompileOptions::new().ok_or("could not create shaderc::CompileOptions")?;
        additional_options.set_target_env(
            shaderc::TargetEnv::Vulkan,
            shaderc::EnvVersion::Vulkan1_3 as u32,
        );
        additional_options.set_target_spirv(shaderc::SpirvVersion::V1_6);
        let include_folder = std::path::PathBuf::from(".");

        additional_options.set_include_callback(move |name, _, _, _| {
            let mut path = include_folder.clone();
            path.push(name);
            if !path.exists() {
                return Err(format!("file does not exist {:?}", path).into());
            }
            let res = std::fs::read_to_string(path.clone()).map_err(|_| {
                format!(
                    "could not read source file {:?} for include {:?}",
                    path, name
                )
            })?;
            Ok(shaderc::ResolvedInclude {
                resolved_name: name.into(),
                content: res.into(),
            })
        });
        // additional_options.set_generate_debug_info();
        self.shader_compiler
            .compile_into_spirv(
                source_text.as_str(),
                shader_kind,
                name_for_errors.as_str(),
                entry_point_name,
                Some(&additional_options),
            )
            .map_err(|x| x.to_string())
            .map(|x| x.as_binary_u8().into())
    }
}
