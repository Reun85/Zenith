#[derive(PartialEq, Eq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(remote = "shaderc::EnvVersion")]
pub enum MyEnvVersion {
    // For Vulkan, use Vulkan's mapping of version numbers to integers.
    // See vulkan.h
    Vulkan1_0,
    Vulkan1_1,
    Vulkan1_2,
    Vulkan1_3,
    OpenGL4_5,
    WebGPU,
}

impl From<shaderc::EnvVersion> for MyEnvVersion {
    fn from(value: shaderc::EnvVersion) -> Self {
        match value {
            shaderc::EnvVersion::Vulkan1_0 => MyEnvVersion::Vulkan1_0,
            shaderc::EnvVersion::Vulkan1_1 => MyEnvVersion::Vulkan1_1,
            shaderc::EnvVersion::Vulkan1_2 => MyEnvVersion::Vulkan1_2,
            shaderc::EnvVersion::Vulkan1_3 => MyEnvVersion::Vulkan1_3,
            shaderc::EnvVersion::OpenGL4_5 => MyEnvVersion::OpenGL4_5,
            shaderc::EnvVersion::WebGPU => MyEnvVersion::WebGPU,
        }
    }
}
#[derive(PartialEq, Eq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(remote = "shaderc::SpirvVersion")]
pub enum MySpirvVersion {
    V1_0,
    V1_1,
    V1_2,
    V1_3,
    V1_4,
    V1_5,
    V1_6,
}
impl From<shaderc::SpirvVersion> for MySpirvVersion {
    fn from(value: shaderc::SpirvVersion) -> Self {
        match value {
            shaderc::SpirvVersion::V1_0 => Self::V1_0,
            shaderc::SpirvVersion::V1_1 => Self::V1_1,
            shaderc::SpirvVersion::V1_2 => Self::V1_2,
            shaderc::SpirvVersion::V1_3 => Self::V1_3,
            shaderc::SpirvVersion::V1_4 => Self::V1_4,
            shaderc::SpirvVersion::V1_5 => Self::V1_5,
            shaderc::SpirvVersion::V1_6 => Self::V1_6,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(remote = "shaderc::ShaderKind")]
pub enum MyShaderKind {
    Vertex,
    Fragment,
    Compute,
    Geometry,
    TessControl,
    TessEvaluation,

    InferFromSource,

    DefaultVertex,
    DefaultFragment,
    DefaultCompute,
    DefaultGeometry,
    DefaultTessControl,
    DefaultTessEvaluation,

    SpirvAssembly,

    RayGeneration,
    AnyHit,
    ClosestHit,
    Miss,
    Intersection,
    Callable,

    DefaultRayGeneration,
    DefaultAnyHit,
    DefaultClosestHit,
    DefaultMiss,
    DefaultIntersection,
    DefaultCallable,

    Task,
    Mesh,

    DefaultTask,
    DefaultMesh,
}
