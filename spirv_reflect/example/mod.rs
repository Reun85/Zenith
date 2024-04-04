use spirv_reflect::ReflectError;
/// Returns the enum variant given at $op if it exists at $idx in $instr.operands
// NOTE: This has to be a macro since enum variants cannot be used for generics.
macro_rules! get_ref_operand_at {
    ($instr:expr, $op:path, $idx:expr) => {
        if $idx >= $instr.operands.len() {
            Err(ReflectError::OperandIndexError(
                $instr.clone(),
                stringify!($op),
                $idx,
                $instr.operands.len(),
            ))
        } else if let $op(val) = &$instr.operands[$idx] {
            Ok(val)
        } else {
            Err(ReflectError::OperandError(
                $instr.clone(),
                stringify!($op),
                $idx,
            ))
        }
    };
}

/// Returns the enum variant given at $op if it exists at $idx in $instr.operands
// NOTE: This has to be a macro since enum variants cannot be used for generics.
macro_rules! get_operand_at {
    ($ops:expr, $op:path, $idx:expr) => {
        get_ref_operand_at!($ops, $op, $idx).map(|v| *v)
    };
}
mod compile;
use rspirv::{
    dr::{Instruction, Operand},
    spirv::Op,
};
use spirv_reflect::DebugNames;

struct LessVerboseInstr<'a, 'b>(&'a Instruction, Option<&'a DebugNames<'b>>);

impl<'a, 'b> std::fmt::Debug for LessVerboseInstr<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.1 {
            None => write!(
                f,
                "Code: {:18}, Type: {:10}, ResId: {:10}, Op: {}",
                format!("{:?}", self.0.class.opcode),
                format!("{:?}", self.0.result_type),
                format!("{:?}", self.0.result_id),
                format!("{:?}", self.0.operands),
            ),
            Some(names) => {
                let name = { names.get_name_from_instr(self.0) };
                write!(
                    f,
                    "Name: {:24}, Code: {:18}, Type: {:10}, ResId: {:10}, Op: {}",
                    format!("{:?}", name),
                    format!("{:?}", self.0.class.opcode),
                    format!("{:?}", self.0.result_type),
                    format!("{:?}", self.0.result_id),
                    format!("{:?}", self.0.operands),
                )
            }
        }
    }
}

fn main() -> Result<(), ReflectError> {
    // let shader_binary =
    //     compile::compile(std::path::PathBuf::from("tests/shaders/vert.vert")).unwrap();
    #[allow(unused_variables)]
    let frag = include_bytes!("./shaders/frag");
    #[allow(unused_variables)]
    let sampler = include_bytes!("./shaders/sampler");
    #[allow(unused_variables)]
    let vert = include_bytes!("./shaders/vert");
    // let reflec = spirv_reflect::Reflection::new_from_spirv(sampler).unwrap();
    let reflec = spirv_reflect::Reflection::new_from_spirv(vert).unwrap();
    let debug_names = reflec.get_debug_names().unwrap();
    // allinstr.iter().for_each(|x| {
    //     println!("{:?}", LessVerboseInstr(x, Some(&debug_names)));
    // });
    // types.iter().for_each(|x| {
    //     println!("{:?}", LessVerboseInstr(x, Some(&debug_names)));
    // });
    // println!("{:#?}", reflec.get_types());
    let types = reflec.get_types()?;
    reflec.0.global_inst_iter().for_each(|x| {
        println!("{:?}", LessVerboseInstr(x, Some(&debug_names)));
    });

    println!("{:#?}", reflec.get_decorations());
    println!("{:#?}", reflec.get_member_decoration());
    Ok(())
}
// mod old {
//     #![allow(dead_code)]
//     #[repr(C)]
//     #[derive(Debug, Clone)]
//     pub struct Vertex {
//         position: cgmath::Vector2<f32>,
//         color: [f32; 3],
//         texture: [f32; 2],
//     }
//     pub type Index = u32;
//     impl Vertex {
//         pub const fn get_binding_description() -> ash::vk::VertexInputBindingDescription {
//             ash::vk::VertexInputBindingDescription {
//                 binding: 0,
//                 stride: std::mem::size_of::<Vertex>() as u32,
//                 input_rate: ash::vk::VertexInputRate::VERTEX,
//             }
//         }
//         pub const fn get_attribute_descriptions() -> [ash::vk::VertexInputAttributeDescription; 3] {
//             [
//                 ash::vk::VertexInputAttributeDescription {
//                     binding: 0,
//                     location: 0,
//                     format: ash::vk::Format::R32G32_SFLOAT,
//                     offset: std::mem::offset_of!(Vertex, position) as u32,
//                 },
//                 ash::vk::VertexInputAttributeDescription {
//                     binding: 0,
//                     location: 1,
//                     format: ash::vk::Format::R32G32B32_SFLOAT,
//                     offset: std::mem::offset_of!(Vertex, color) as u32,
//                 },
//                 ash::vk::VertexInputAttributeDescription {
//                     binding: 0,
//                     location: 2,
//                     format: ash::vk::Format::R32G32_SFLOAT,
//                     offset: std::mem::offset_of!(Vertex, texture) as u32,
//                 },
//             ]
//         }
//     }
//
//     // This type matches
//     use cgmath::{Matrix4, SquareMatrix, Vector2};
//     #[repr(C, align(16))]
//     struct UniformBufferObject {
//         eye: Vector2<f32>,
//         model: Matrix4<f32>,
//         view: Matrix4<f32>,
//         proj: Matrix4<f32>,
//     }
//
//     impl Default for UniformBufferObject {
//         fn default() -> Self {
//             Self {
//                 eye: Vector2::new(0.0, 0.0),
//                 model: Matrix4::identity(),
//                 view: Matrix4::identity(),
//                 proj: Matrix4::identity(),
//             }
//         }
//     }
//     impl UniformBufferObject {
//         const fn memory_offset() -> [usize; 4] {
//             [0, 16, 80, 144]
//         }
//         unsafe fn copy_to_memory_address(&self, dst: *mut std::ffi::c_void) {
//             std::ptr::copy_nonoverlapping(&self.eye, dst.offset(0) as *mut Vector2<f32>, 1);
//             std::ptr::copy_nonoverlapping(&self.model, dst.offset(16) as *mut Matrix4<f32>, 1);
//             std::ptr::copy_nonoverlapping(&self.view, dst.offset(80) as *mut Matrix4<f32>, 1);
//             std::ptr::copy_nonoverlapping(&self.proj, dst.offset(144) as *mut Matrix4<f32>, 1);
//         }
//
//         const fn size() -> usize {
//             std::mem::size_of::<cgmath::Vector4<f32>>() + std::mem::size_of::<Matrix4<f32>>() * 3
//         }
//     }
//     impl UniformBufferObject {
//         pub const fn get_descriptor_set_layout_binding() -> ash::vk::DescriptorSetLayoutBinding {
//             ash::vk::DescriptorSetLayoutBinding {
//                 binding: 0,
//                 descriptor_type: ash::vk::DescriptorType::UNIFORM_BUFFER,
//                 descriptor_count: 1,
//                 stage_flags: ash::vk::ShaderStageFlags::VERTEX,
//                 p_immutable_samplers: std::ptr::null(),
//             }
//         }
//     }
//
//     struct Sampler2D;
//
//     impl Sampler2D {
//         pub const fn get_descriptor_set_layout_binding() -> ash::vk::DescriptorSetLayoutBinding {
//             ash::vk::DescriptorSetLayoutBinding {
//                 binding: 1,
//                 descriptor_type: ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
//                 descriptor_count: 1,
//                 stage_flags: ash::vk::ShaderStageFlags::FRAGMENT,
//                 p_immutable_samplers: std::ptr::null(),
//             }
//         }
//     }
// }
