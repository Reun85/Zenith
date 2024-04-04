//! Adding functionality to library [rspirv_reflect](https://crates.io/crates/rspirv_reflect)
//! Basic SPIR-V reflection library to extract binding information
//!
//! ```no_run
//! # let spirv_blob: &[u8] = todo!();
//! let info = rspirv_reflect::Reflection::new_from_spirv(&spirv_blob).expect("Invalid SPIR-V");
//! dbg!(info.get_descriptor_sets().expect("Failed to extract descriptor bindings"));
//! ```

#![deny(clippy::correctness, clippy::complexity)]
#![warn(
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
    clippy::suspicious,
    clippy::style
)]

use rspirv::dr::{Instruction, Module, Operand};
use rspirv::spirv::StorageClass;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::num::TryFromIntError;
use std::rc::Rc;
use thiserror::Error;

pub mod types;
pub use rspirv;
pub use rspirv::spirv;

pub struct Reflection(pub Module);

/// Mostly a collection of errors caused by a potentially invalid SPIR-V module, or invalid usage
/// of said file.
#[derive(Error, Debug)]
pub enum ReflectError {
    // NOTE: Instructions are stored as string because they cannot be cloned,
    // and storing a reference means the shader must live at least as long as
    // the error bubbling up, which is generally impossible.
    #[error("{0:?} missing binding decoration")]
    MissingBindingDecoration(Instruction),
    #[error("{0:?} missing set decoration")]
    MissingSetDecoration(Instruction),
    #[error("Expecting operand {1} in position {2} for instruction {0:?}")]
    OperandError(Instruction, &'static str, usize),
    #[error("Expecting operand {1} in position {2} but instruction {0:?} has only {3} operands")]
    OperandIndexError(Instruction, &'static str, usize, usize),
    #[error("OpVariable {0:?} lacks a return type")]
    VariableWithoutReturnType(Instruction),
    #[error("Unknown storage class {0:?}")]
    UnknownStorageClass(spirv::StorageClass),
    #[error("Unknown struct (missing Block or BufferBlock annotation): {0:?}")]
    UnknownStruct(Instruction),
    #[error("Unknown value {1} for `sampled` field: {0:?}")]
    ImageSampledFieldUnknown(Instruction, u32),
    #[error("Unhandled OpType instruction {0:?}")]
    UnhandledTypeInstruction(Instruction),
    #[error("{0:?} does not generate a result")]
    MissingResultId(Instruction),
    #[error("No instruction assigns to {0:?}")]
    UnassignedResultId(u32),
    #[error("rspirv reflect lacks module header")]
    MissingHeader,
    #[error("rspirv reflect lacks `OpMemoryModel`")]
    MissingMemoryModel,
    #[error("Accidentally binding global parameter buffer. Global variables are currently not supported in HLSL")]
    BindingGlobalParameterBuffer,
    #[error("Only one push constant block can be defined per shader entry")]
    TooManyPushConstants,
    #[error("SPIR-V parse error")]
    ParseError(#[from] rspirv::binary::ParseState),
    #[error("OpTypeInt cannot have width {0}")]
    UnexpectedIntWidth(u32),
    #[error(
        "Invalid or unimplemented combination of AddressingModel {0:?} and StorageClass {1:?}"
    )]
    InvalidAddressingModelAndStorageClass(spirv::AddressingModel, spirv::StorageClass),
    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),
    #[error("Missing debug name for instruction {0:?}")]
    MissingDebugName(Instruction),

    #[error("Invalid execution mode, expected one of {1:?}, got: {0:?}")]
    ExecutionModeIsIncorrect(
        rspirv::spirv::ExecutionMode,
        Vec<rspirv::spirv::ExecutionMode>,
    ),
    #[error("Missing execution mode")]
    ExecutionModeMissing(),
    #[error("A Type Id was referenced before it was defined")]
    UnresolvedTypeId(u32),
    #[error("Invalid Inner type for instruction {0:?}")]
    InvalidInnerType(Instruction),
}

type Result<V, E = ReflectError> = ::std::result::Result<V, E>;

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

type Id = u32;
type MemberId = u32;
type TypeId = u32;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugNames<'a> {
    pub name: BTreeMap<Id, &'a str>,
    pub member_name: BTreeMap<Id, BTreeMap<MemberId, &'a str>>,
}

impl<'a> DebugNames<'a> {
    #[must_use]
    pub fn get_name(&self, id: Id) -> Option<&&'a str> {
        self.name.get(&id)
    }
    #[must_use]
    pub fn get_member_name(&self, id: Id, member_id: MemberId) -> Option<&&'a str> {
        self.member_name.get(&id)?.get(&member_id)
    }
    #[must_use]
    pub fn get_name_from_instr(&self, instr: &Instruction) -> Option<&&'a str> {
        use spirv::Op;
        match instr.class.opcode {
            Op::MemberName | Op::MemberDecorate => get_operand_at!(instr, Operand::IdRef, 0)
                .map_or(None, |id| {
                    get_operand_at!(instr, Operand::LiteralBit32, 1).map_or_else(
                        |_| self.name.get(&id),
                        |bit| self.member_name.get(&id).and_then(|inner| inner.get(&bit)),
                    )
                }),

            _ => instr
                .result_id
                .or_else(|| get_operand_at!(instr, Operand::IdRef, 0).ok())
                .and_then(|x| self.name.get(&x)),
        }
    }
}

/// These are bit-exact with ash and the Vulkan specification,
/// they're mirrored here to prevent a dependency on ash
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
#[non_exhaustive]
pub enum DescriptorType {
    Sampler = 0,
    CombinedImageSampler = 1,
    SampledImage = 2,
    StorageImage = 3,
    UniformTexelBuffer = 4,
    StorageTexelBuffer = 5,
    UniformBuffer = 6,
    StorageBuffer = 7,
    UniformBufferDynamic = 8,
    StorageBufferDynamic = 9,
    InputAttachment = 10,

    InlineUniformBufferExt = 1_000_138_000,
    AccelerationStructureKhr = 1_000_150_000,
    AccelerationStructureNv = 1_000_165_000,
}

// /// These are bit-exact with ash and the Vulkan specification,
// /// they're mirrored here to prevent a dependency on ash
// #[derive(Copy, Clone, Eq, PartialEq)]
// #[repr(transparent)]
// pub struct DescriptorType(pub u32);
//
// impl DescriptorType {
//     pub const SAMPLER: Self = Self(0);
//     pub const COMBINED_IMAGE_SAMPLER: Self = Self(1);
//     pub const SAMPLED_IMAGE: Self = Self(2);
//     pub const STORAGE_IMAGE: Self = Self(3);
//     pub const UNIFORM_TEXEL_BUFFER: Self = Self(4);
//     pub const STORAGE_TEXEL_BUFFER: Self = Self(5);
//     pub const UNIFORM_BUFFER: Self = Self(6);
//     pub const STORAGE_BUFFER: Self = Self(7);
//     pub const UNIFORM_BUFFER_DYNAMIC: Self = Self(8);
//     pub const STORAGE_BUFFER_DYNAMIC: Self = Self(9);
//     pub const INPUT_ATTACHMENT: Self = Self(10);
//
//     pub const INLINE_UNIFORM_BLOCK_EXT: Self = Self(1_000_138_000);
//     pub const ACCELERATION_STRUCTURE_KHR: Self = Self(1_000_150_000);
//     pub const ACCELERATION_STRUCTURE_NV: Self = Self(1_000_165_000);
// }

impl std::fmt::Debug for DescriptorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(unreachable_patterns)]
        f.write_str(match self {
            Self::Sampler => "SAMPLER",
            Self::CombinedImageSampler => "COMBINED_IMAGE_SAMPLER",
            Self::SampledImage => "SAMPLED_IMAGE",
            Self::StorageImage => "STORAGE_IMAGE",
            Self::UniformTexelBuffer => "UNIFORM_TEXEL_BUFFER",
            Self::StorageTexelBuffer => "STORAGE_TEXEL_BUFFER",
            Self::UniformBuffer => "UNIFORM_BUFFER",
            Self::StorageBuffer => "STORAGE_BUFFER",
            Self::UniformBufferDynamic => "UNIFORM_BUFFER_DYNAMIC",
            Self::StorageBufferDynamic => "STORAGE_BUFFER_DYNAMIC",
            Self::InputAttachment => "INPUT_ATTACHMENT",
            Self::InlineUniformBufferExt => "INLINE_UNIFORM_BLOCK_EXT",
            Self::AccelerationStructureKhr => "ACCELERATION_STRUCTURE_KHR",
            Self::AccelerationStructureNv => "ACCELERATION_STRUCTURE_NV",
            _ => "(UNDEFINED)",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingCount {
    /// A single resource binding.
    ///
    /// # Example
    /// ```hlsl
    /// StructuredBuffer<uint>
    /// ```
    One,
    /// Predetermined number of resource bindings.
    ///
    /// # Example
    /// ```hlsl
    /// StructuredBuffer<uint> myBinding[4]
    /// ```
    StaticSized(usize),
    /// Variable number of resource bindings (usually dubbed "bindless").
    ///
    /// Count is determined in `vkDescriptorSetLayoutBinding`. No other bindings should follow in this set.
    ///
    /// # Example
    /// ```hlsl
    /// StructuredBuffer<uint> myBinding[]
    /// ```
    Unbounded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescriptorInfo {
    pub ty: DescriptorType,
    pub binding_count: BindingCount,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Decoration {
    pub binding: Option<u32>,
    // pub set: Option<u32>,
    pub location: Option<u32>,
    pub offset: Option<u32>,
    pub array_stride: Option<u32>,
    pub descriptor_set: Option<u32>,
    pub nonwritable: bool,
    pub nonreadable: bool,
    pub builtin: Option<spirv::BuiltIn>,
    pub input_attachment_index: Option<u32>,
    pub push_constant: Option<PushConstantInfo>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PushConstantInfo {
    pub offset: u32,
    pub size: u32,
}

impl Reflection {
    // New from module.
    #[must_use]
    pub const fn new(module: Module) -> Self {
        Self(module)
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if the supplied binary is not valid SPIR-V Code.
    // Parse from a SPIR-V binary.
    pub fn new_from_spirv(code: &[u8]) -> Result<Self> {
        let module = rspirv::dr::load_bytes(code)?;
        Ok(Self::new(module))
    }
    // Parse from a SPIR-V binary.
    pub fn new_from_spirv_words(words: &[u32]) -> Result<Self> {
        let module = rspirv::dr::load_words(words)?;
        Ok(Self::new(module))
    }

    pub fn get_memory_model(&self) -> Result<spirv::MemoryModel> {
        let instr = self
            .0
            .memory_model
            .clone()
            .ok_or(ReflectError::MissingMemoryModel)?;
        let _addressingmode = get_operand_at!(instr, Operand::AddressingModel, 0);
        get_operand_at!(instr, Operand::MemoryModel, 1)
    }

    pub fn get_debug_names(&self) -> DebugNames<'_> {
        let (names, member_names): (Vec<_>, Vec<_>) = self
            .0
            .debug_names
            .iter()
            .filter(|&i| {
                i.class.opcode == spirv::Op::Name || i.class.opcode == spirv::Op::MemberName
            })
            .map(|x| -> Result<(u32, Option<u32>, &str)> {
                match x.class.opcode {
                    spirv::Op::Name => {
                        let id = get_operand_at!(x, Operand::IdRef, 0)?;
                        let name = get_ref_operand_at!(x, Operand::LiteralString, 1)?;
                        Ok((id, None, name.as_str()))
                    }
                    spirv::Op::MemberName => {
                        let id = get_operand_at!(x, Operand::IdRef, 0)?;
                        let member_id = get_operand_at!(x, Operand::LiteralBit32, 1)?;
                        let name = get_ref_operand_at!(x, Operand::LiteralString, 2)?;
                        Ok((id, Some(member_id), name.as_str()))
                    }

                    _ => unreachable!(),
                }
            })
            .filter_map(std::result::Result::ok)
            .filter(
                |x| // NOTE: If a debug name is "" its usually built in. not useful
                !x.2.is_empty(),
            )
            .collect::<Vec<_>>()
            .into_iter()
            .partition(|x| x.1.is_none());
        let names = names.into_iter().map(|(id, _, name)| (id, name)).collect();
        let member_names = {
            let mut outer_map: BTreeMap<u32, BTreeMap<u32, &str>> = BTreeMap::new();

            for (outer_key, inner_key, value) in member_names {
                outer_map
                    .entry(outer_key)
                    .or_default()
                    .insert(inner_key.unwrap(), value);
            }

            outer_map
        };
        DebugNames {
            name: names,
            member_name: member_names,
        }
    }

    pub fn get_all_variables<'a>(&'a self) -> Result<Vec<(Id, StorageClass)>> {
        self.get_all_variables_iter().collect::<Result<Vec<_>>>()
    }
    pub fn get_all_variables_iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = std::result::Result<(u32, rspirv::spirv::StorageClass), ReflectError>> + 'a
    {
        self.0
            .types_global_values
            .iter()
            .filter(|inst| match inst.class.opcode {
                spirv::Op::Variable => true,
                _ => false,
            })
            .map(|inst| {
                let cls = get_operand_at!(inst, Operand::StorageClass, 0);
                let result_id = inst
                    .result_id
                    .ok_or(ReflectError::MissingResultId(inst.clone()))?;

                match cls {
                    Ok(cls) => Ok((result_id, cls)),
                    Err(e) => Err(e),
                }
            })
    }
    pub fn get_type_of_variable(&self, id: Id) -> Result<TypeId> {
        let inst = self
            .0
            .types_global_values
            .iter()
            .find(|inst| inst.result_id == Some(id))
            .ok_or(ReflectError::UnassignedResultId(id))?;
        match inst.class.opcode {
            spirv::Op::Variable => {
                // For some reason variable types are typepointers?
                let type_pointer_id = inst
                    .result_type
                    .ok_or(ReflectError::VariableWithoutReturnType(inst.clone()))?;
                let type_inst = self
                    .0
                    .types_global_values
                    .iter()
                    .find(|inst2| inst2.result_id.is_some_and(|val| val == type_pointer_id))
                    .ok_or(ReflectError::UnresolvedTypeId(type_pointer_id))?;
                // TODO: What if storageclass of variable and type isn't the same?
                debug_assert!(
                    get_operand_at!(inst, Operand::StorageClass, 0)?
                        == get_operand_at!(type_inst, Operand::StorageClass, 0)?
                );

                get_operand_at!(type_inst, Operand::IdRef, 1)
            }
            _ => Err(ReflectError::UnassignedResultId(id)),
        }
    }
    // pub fn get_all_types(&self) -> BTreeMap<u32, TypeInfo> {}
    pub fn get_all_variables_with_storage_class<'a>(
        &'a self,
        class: StorageClass,
    ) -> Result<Vec<Id>> {
        self.get_all_variables_iter()
            .filter_map(|x| match x {
                Ok((id, cls)) if cls == class => Some(Ok(id)),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect()
    }
    pub fn get_types(&self) -> Result<BTreeMap<TypeId, Rc<types::Type>>> {
        let mut types = BTreeMap::new();
        for inst in self.0.types_global_values.iter() {
            match inst.class.opcode {
                spirv::Op::TypeInt => {
                    let bits = get_operand_at!(inst, Operand::LiteralBit32, 0)? as u16;
                    let issigned = get_operand_at!(inst, Operand::LiteralBit32, 1)? == 1;
                    let type_info = Rc::new(types::Type::Int(types::Int { bits, issigned }));
                    let result_id = inst
                        .result_id
                        .ok_or(ReflectError::MissingResultId(inst.clone()))?;
                    types.insert(result_id, type_info);
                }
                spirv::Op::TypeFloat => {
                    let bits = get_operand_at!(inst, Operand::LiteralBit32, 0)? as u16;
                    let type_info = Rc::new(types::Type::Float(types::Float { bits }));
                    let result_id = inst
                        .result_id
                        .ok_or(ReflectError::MissingResultId(inst.clone()))?;
                    types.insert(result_id, type_info);
                }
                spirv::Op::TypeVector => {
                    let typ = get_operand_at!(inst, Operand::IdRef, 0)?;
                    let inner = types
                        .get(&typ)
                        .ok_or(ReflectError::UnresolvedTypeId(typ))?
                        .clone();

                    let len = get_operand_at!(inst, Operand::LiteralBit32, 1)? as u16;
                    let type_info = Rc::new(types::Type::Vector(types::Vector {
                        inner_type: inner,
                        size: len.into(),
                    }));
                    let result_id = inst
                        .result_id
                        .ok_or(ReflectError::MissingResultId(inst.clone()))?;
                    types.insert(result_id, type_info);
                }
                spirv::Op::TypeMatrix => {
                    let typ = get_operand_at!(inst, Operand::IdRef, 0)?;
                    let inner = types
                        .get(&typ)
                        .ok_or(ReflectError::UnresolvedTypeId(typ))?
                        .clone();

                    let len = get_operand_at!(inst, Operand::LiteralBit32, 1)? as u16;
                    let type_info = Rc::new(types::Type::Mat(types::Mat {
                        inner_type: inner,
                        size: len.into(),
                    }));
                    let result_id = inst
                        .result_id
                        .ok_or(ReflectError::MissingResultId(inst.clone()))?;
                    types.insert(result_id, type_info);
                }
                spirv::Op::TypeArray => {
                    let typ = get_operand_at!(inst, Operand::IdRef, 0)?;
                    let inner = types
                        .get(&typ)
                        .ok_or(ReflectError::UnresolvedTypeId(typ))?
                        .clone();

                    let len_id = get_operand_at!(inst, Operand::IdRef, 1)?;

                    let len = self
                        .0
                        .types_global_values
                        .iter()
                        .find_map(|i| {
                            if i.result_id == Some(len_id) {
                                match i.class.opcode {
                                    spirv::Op::Constant => {
                                        get_operand_at!(i, Operand::LiteralBit32, 0).ok()
                                    }
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        })
                        .ok_or(ReflectError::UnassignedResultId(len_id))?
                        as u16;

                    let type_info = Rc::new(types::Type::Array(types::Array {
                        inner_type: inner,
                        len,
                    }));
                    let result_id = inst
                        .result_id
                        .ok_or(ReflectError::MissingResultId(inst.clone()))?;
                    types.insert(result_id, type_info);
                }
                spirv::Op::TypeRuntimeArray => {
                    let typ = get_operand_at!(inst, Operand::IdRef, 0)?;
                    let inner = types
                        .get(&typ)
                        .ok_or(ReflectError::UnresolvedTypeId(typ))?
                        .clone();

                    let type_info = Rc::new(types::Type::RunTimeArray(types::RunTimeArray {
                        inner_type: inner,
                    }));
                    let result_id = inst
                        .result_id
                        .ok_or(ReflectError::MissingResultId(inst.clone()))?;
                    types.insert(result_id, type_info);
                }
                spirv::Op::TypeStruct => {
                    let fields = inst
                        .operands
                        .iter()
                        .map(|op| -> Result<Rc<types::Type>> {
                            match op {
                                Operand::IdRef(typ) => {
                                    let inner = types
                                        .get(&typ)
                                        .ok_or(ReflectError::UnresolvedTypeId(*typ))?
                                        .clone();
                                    Ok(inner)
                                }
                                _ => Err(ReflectError::InvalidInnerType(inst.clone())),
                            }
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let type_info = Rc::new(types::Type::Struct(types::Struct {
                        fields,
                        base_alignment: 16,
                        block_decor: false,
                    }));
                    let result_id = inst
                        .result_id
                        .ok_or(ReflectError::MissingResultId(inst.clone()))?;
                    types.insert(result_id, type_info);
                }
                _ => {}
            }
        }
        Ok(types)
    }

    pub fn get_decorations(&self) -> BTreeMap<Id, Decoration> {
        let mut res = BTreeMap::new();
        for inst in self.0.annotations.iter() {
            match inst.class.opcode {
                spirv::Op::Decorate => {
                    let target_id = get_operand_at!(inst, Operand::IdRef, 0).unwrap();
                    let decoration = get_operand_at!(inst, Operand::Decoration, 1).unwrap();
                    // let mut dec: &Decoration = res.get(&target_id).unwrap_or_default();
                    let dec = res.entry(target_id).or_insert_with(Decoration::default);
                    match decoration {
                        spirv::Decoration::Binding => {
                            dec.binding =
                                Some(get_operand_at!(inst, Operand::LiteralBit32, 2).unwrap())
                        }
                        spirv::Decoration::Location => {
                            dec.location =
                                Some(get_operand_at!(inst, Operand::LiteralBit32, 2).unwrap())
                        }
                        spirv::Decoration::Offset => {
                            dec.offset =
                                Some(get_operand_at!(inst, Operand::LiteralBit32, 2).unwrap())
                        }
                        spirv::Decoration::ArrayStride => {
                            dec.array_stride =
                                Some(get_operand_at!(inst, Operand::LiteralBit32, 2).unwrap())
                        }
                        spirv::Decoration::DescriptorSet => {
                            dec.descriptor_set =
                                Some(get_operand_at!(inst, Operand::LiteralBit32, 2).unwrap())
                        }
                        spirv::Decoration::NonWritable => {
                            dec.nonwritable = true;
                        }
                        spirv::Decoration::NonReadable => {
                            dec.nonreadable = true;
                        }
                        spirv::Decoration::BuiltIn => {
                            dec.builtin = Some(get_operand_at!(inst, Operand::BuiltIn, 2).unwrap())
                        }
                        spirv::Decoration::InputAttachmentIndex => {
                            dec.input_attachment_index =
                                Some(get_operand_at!(inst, Operand::LiteralBit32, 2).unwrap())
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        res
    }
    pub fn get_member_decoration(&self) -> BTreeMap<Id, BTreeMap<MemberId, Decoration>> {
        let mut res = BTreeMap::new();
        for inst in self.0.annotations.iter() {
            match inst.class.opcode {
                spirv::Op::MemberDecorate => {
                    let target_id = get_operand_at!(inst, Operand::IdRef, 0).unwrap();
                    let member_id = get_operand_at!(inst, Operand::LiteralBit32, 1).unwrap();
                    let decoration = get_operand_at!(inst, Operand::Decoration, 2).unwrap();
                    let dec = res
                        .entry(target_id)
                        .or_insert_with(BTreeMap::new)
                        .entry(member_id)
                        .or_insert_with(Decoration::default);
                    match decoration {
                        spirv::Decoration::Binding => {
                            dec.binding =
                                Some(get_operand_at!(inst, Operand::LiteralBit32, 3).unwrap())
                        }
                        spirv::Decoration::Location => {
                            dec.location =
                                Some(get_operand_at!(inst, Operand::LiteralBit32, 3).unwrap())
                        }
                        spirv::Decoration::Offset => {
                            dec.offset =
                                Some(get_operand_at!(inst, Operand::LiteralBit32, 3).unwrap())
                        }
                        spirv::Decoration::ArrayStride | spirv::Decoration::MatrixStride => {
                            dec.array_stride =
                                Some(get_operand_at!(inst, Operand::LiteralBit32, 3).unwrap())
                        }
                        spirv::Decoration::DescriptorSet => {
                            dec.descriptor_set =
                                Some(get_operand_at!(inst, Operand::LiteralBit32, 3).unwrap())
                        }
                        spirv::Decoration::NonWritable => {
                            dec.nonwritable = true;
                        }
                        spirv::Decoration::NonReadable => {
                            dec.nonreadable = true;
                        }
                        spirv::Decoration::BuiltIn => {
                            dec.builtin = Some(get_operand_at!(inst, Operand::BuiltIn, 3).unwrap())
                        }
                        spirv::Decoration::InputAttachmentIndex => {
                            dec.input_attachment_index =
                                Some(get_operand_at!(inst, Operand::LiteralBit32, 3).unwrap())
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        res
    }

    /// Only works if the only `ExecutionMode` is `LocalSize`, that is, the `SPIR-V` is a `compute` shader.
    pub fn get_compute_group_size(&self) -> Result<(u32, u32, u32)> {
        for inst in self.0.global_inst_iter() {
            if inst.class.opcode == spirv::Op::ExecutionMode {
                use rspirv::dr::Operand::{ExecutionMode, LiteralBit32};
                let exmode = get_operand_at!(inst, ExecutionMode, 1)?;
                match exmode {
                    spirv::ExecutionMode::LocalSize | spirv::ExecutionMode::LocalSizeHint => {
                        let x = get_operand_at!(inst, LiteralBit32, 2)?;
                        let y = get_operand_at!(inst, LiteralBit32, 3)?;
                        let z = get_operand_at!(inst, LiteralBit32, 4)?;
                        return Ok((x, y, z));
                    }
                    _ => {
                        return Err(ReflectError::ExecutionModeIsIncorrect(
                            exmode,
                            vec![
                                rspirv::spirv::ExecutionMode::LocalSize,
                                rspirv::spirv::ExecutionMode::LocalSizeHint,
                            ],
                        ))
                    }
                }
            }
        }
        Err(ReflectError::ExecutionModeMissing())
    }

    /// Returns the descriptor type for a given variable `type_id`
    fn get_descriptor_type_for_var(
        &self,
        type_id: u32,
        storage_class: spirv::StorageClass,
    ) -> Result<DescriptorInfo> {
        let type_instruction =
            find_instructions_assigning_to_id(&self.0.types_global_values, type_id)?;
        self.get_descriptor_type(type_instruction, storage_class)
    }

    /// Returns the descriptor type for a given `OpType*` `Instruction`
    fn get_descriptor_type(
        &self,
        type_instruction: &Instruction,
        storage_class: spirv::StorageClass,
    ) -> Result<DescriptorInfo> {
        let annotations = type_instruction.result_id.map_or(Ok(vec![]), |result_id| {
            filter_annotations_with_id(&self.0.annotations, result_id)
        })?;

        // Weave with recursive types
        match type_instruction.class.opcode {
            spirv::Op::TypeVector => {}
            spirv::Op::TypeArray => {
                /*
                %7 = OpTypeVector %6 4                      ; vec4
                %8 = OpTypeInt 32 0                         ; 32-bit int, sign-less
                %9 = OpConstant %8 5                        ; const %8:u32 = 5
                %10 = OpTypeArray %7 %9                     ; vec4[5]
                            */
                let element_type_id = get_operand_at!(type_instruction, Operand::IdRef, 0)?;
                let num_elements_id = get_operand_at!(type_instruction, Operand::IdRef, 1)?;
                let num_elements = find_instructions_assigning_to_id(
                    &self.0.types_global_values,
                    num_elements_id,
                )?;
                assert_eq!(
                    num_elements.class.opcode,
                    spirv::Op::Constant,
                    "SPIR-V arrays can only be initialised with a constant length."
                );
                let num_elements_ty = find_instructions_assigning_to_id(
                    &self.0.types_global_values,
                    num_elements.result_type.unwrap(),
                )?;
                // Array size can be any width, any signedness
                assert_eq!(num_elements_ty.class.opcode, spirv::Op::TypeInt);
                let num_elements = match get_operand_at!(num_elements_ty, Operand::LiteralBit32, 0)?
                {
                    32 => get_operand_at!(num_elements, Operand::LiteralBit32, 0)?.try_into()?,
                    64 => get_operand_at!(num_elements, Operand::LiteralBit64, 0)?.try_into()?,
                    x => return Err(ReflectError::UnexpectedIntWidth(x)),
                };
                assert!(num_elements >= 1);
                return Ok(DescriptorInfo {
                    binding_count: BindingCount::StaticSized(num_elements),
                    ..self.get_descriptor_type_for_var(element_type_id, storage_class)?
                });
            }
            spirv::Op::TypeRuntimeArray => {
                // vec4 array[] ; unknown size
                let element_type_id = get_operand_at!(type_instruction, Operand::IdRef, 0)?;
                return Ok(DescriptorInfo {
                    binding_count: BindingCount::Unbounded,
                    ..self.get_descriptor_type_for_var(element_type_id, storage_class)?
                });
            }
            spirv::Op::TypePointer => {
                // vec4* ptr
                let ptr_storage_class =
                    get_operand_at!(type_instruction, Operand::StorageClass, 0)?;
                let element_type_id = get_operand_at!(type_instruction, Operand::IdRef, 1)?;
                assert_eq!(storage_class, ptr_storage_class);
                return self.get_descriptor_type_for_var(element_type_id, storage_class);
            }
            spirv::Op::TypeSampledImage => {
                // Essentially any sample2D, sample3D, sampleCube etc.
                /*
                %image_type = OpTypeImage %float 2D 1 0 0 1 Unknown
                %sampled_image = OpTypeSampledImage %image_type
                                    */
                let element_type_id = get_operand_at!(type_instruction, Operand::IdRef, 0)?;

                // get %image_type
                let image_instruction = find_instructions_assigning_to_id(
                    &self.0.types_global_values,
                    element_type_id,
                )?;

                let descriptor = self.get_descriptor_type(image_instruction, storage_class)?;

                let dim = get_operand_at!(image_instruction, Operand::Dim, 1)?;
                assert_ne!(dim, spirv::Dim::DimSubpassData);

                return Ok(if dim == spirv::Dim::DimBuffer {
                    if descriptor.ty != DescriptorType::UniformTexelBuffer
                        && descriptor.ty != DescriptorType::StorageTexelBuffer
                    {
                        todo!("Unexpected sampled image type {:?}", descriptor.ty)
                    }
                    descriptor
                } else {
                    DescriptorInfo {
                        ty: DescriptorType::CombinedImageSampler,
                        ..descriptor
                    }
                });
            }
            _ => {}
        }

        let descriptor_type = match type_instruction.class.opcode {
            spirv::Op::TypeSampler => DescriptorType::Sampler,
            spirv::Op::TypeImage => {
                let dim = get_operand_at!(type_instruction, Operand::Dim, 1)?;

                const IMAGE_SAMPLED: u32 = 1;
                const IMAGE_STORAGE: u32 = 2;

                // TODO: Should this be modeled as an enum in rspirv??
                let sampled = get_operand_at!(type_instruction, Operand::LiteralBit32, 5)?;

                if dim == spirv::Dim::DimBuffer {
                    if sampled == IMAGE_SAMPLED {
                        DescriptorType::UniformTexelBuffer
                    } else if sampled == IMAGE_STORAGE {
                        DescriptorType::StorageTexelBuffer
                    } else {
                        return Err(ReflectError::ImageSampledFieldUnknown(
                            type_instruction.clone(),
                            sampled,
                        ));
                    }
                } else if dim == spirv::Dim::DimSubpassData {
                    DescriptorType::InputAttachment
                } else if sampled == IMAGE_SAMPLED {
                    DescriptorType::SampledImage
                } else if sampled == IMAGE_STORAGE {
                    DescriptorType::StorageImage
                } else {
                    return Err(ReflectError::ImageSampledFieldUnknown(
                        type_instruction.clone(),
                        sampled,
                    ));
                }
            }
            spirv::Op::TypeStruct => {
                let mut is_uniform_buffer = false;
                let mut is_storage_buffer = false;

                for annotation in annotations {
                    for operand in &annotation.operands {
                        if let Operand::Decoration(decoration) = operand {
                            match decoration {
                                spirv::Decoration::Block => is_uniform_buffer = true,
                                spirv::Decoration::BufferBlock => is_storage_buffer = true,
                                _ => { /* println!("Unhandled decoration {:?}", decoration) */ }
                            }
                        }
                    }
                }

                let version = self
                    .0
                    .header
                    .as_ref()
                    .ok_or(ReflectError::MissingHeader)?
                    .version();

                if version <= (1, 3) && is_storage_buffer {
                    // BufferBlock is still support in 1.3 exactly.
                    DescriptorType::StorageBuffer
                } else if version >= (1, 3) {
                    // From 1.3, StorageClass is supported.
                    assert!(
                        !is_storage_buffer,
                        "BufferBlock decoration is obsolete in SPIRV > 1.3"
                    );
                    assert!(
                        is_uniform_buffer,
                        "Struct requires Block annotation in SPIRV > 1.3"
                    );
                    match storage_class {
                        spirv::StorageClass::Uniform | spirv::StorageClass::UniformConstant => {
                            DescriptorType::UniformBuffer
                        }
                        spirv::StorageClass::StorageBuffer => DescriptorType::StorageBuffer,
                        _ => return Err(ReflectError::UnknownStorageClass(storage_class)),
                    }
                } else if is_uniform_buffer {
                    DescriptorType::UniformBuffer
                } else {
                    return Err(ReflectError::UnknownStruct(type_instruction.clone()));
                }
            }
            // TODO: spirv_reflect translates nothing to {UNIFORM,STORAGE}_BUFFER_DYNAMIC
            spirv::Op::TypeAccelerationStructureKHR => DescriptorType::AccelerationStructureKhr,
            _ => {
                return Err(ReflectError::UnhandledTypeInstruction(
                    type_instruction.clone(),
                ))
            }
        };

        Ok(DescriptorInfo {
            ty: descriptor_type,
            binding_count: BindingCount::One,
            name: "".to_string(),
        })
    }

    /// Returns a nested mapping, where the first level maps descriptor set indices (register spaces)
    /// and the second level maps descriptor binding indices (registers) to descriptor information.
    pub fn get_descriptor_sets(&self) -> Result<BTreeMap<u32, BTreeMap<u32, DescriptorInfo>>> {
        let mut unique_sets = BTreeMap::new();
        let reflect = &self.0;

        let uniform_variables = reflect
            .types_global_values
            .iter()
            .filter(|i| i.class.opcode == spirv::Op::Variable)
            .filter_map(|i| {
                let cls = get_operand_at!(i, Operand::StorageClass, 0);
                match cls {
                    Ok(cls)
                        if cls == spirv::StorageClass::Uniform
                            || cls == spirv::StorageClass::UniformConstant
                            || cls == spirv::StorageClass::StorageBuffer =>
                    {
                        Some(Ok(i))
                    }
                    Err(e) => Some(Err(e)),
                    _ => None,
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let names = reflect
            .debug_names
            .iter()
            .filter(|i| i.class.opcode == spirv::Op::Name)
            .map(|i| -> Result<(u32, String)> {
                let element_type_id = get_operand_at!(i, Operand::IdRef, 0)?;
                let name = get_ref_operand_at!(i, Operand::LiteralString, 1)?;
                Ok((element_type_id, name.clone()))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;

        for var in uniform_variables {
            if let Some(var_id) = var.result_id {
                let annotations = filter_annotations_with_id(&reflect.annotations, var_id)?;

                // TODO: Can also define these as mut
                let (set, binding) = annotations.iter().filter(|a| a.operands.len() >= 3).fold(
                    (None, None),
                    |state, a| {
                        if let Operand::Decoration(d) = a.operands[1] {
                            if let Operand::LiteralBit32(i) = a.operands[2] {
                                match d {
                                    spirv::Decoration::DescriptorSet => {
                                        assert!(state.0.is_none(), "Set already has a value!");
                                        return (Some(i), state.1);
                                    }
                                    spirv::Decoration::Binding => {
                                        assert!(state.1.is_none(), "Binding already has a value!");
                                        return (state.0, Some(i));
                                    }
                                    _ => {}
                                }
                            }
                        }
                        state
                    },
                );

                let set = set.ok_or_else(|| ReflectError::MissingSetDecoration(var.clone()))?;
                let binding =
                    binding.ok_or_else(|| ReflectError::MissingBindingDecoration(var.clone()))?;

                let current_set = /* &mut */ unique_sets
                    .entry(set)
                    .or_insert_with(BTreeMap::<u32, DescriptorInfo>::new);

                let storage_class = get_operand_at!(var, Operand::StorageClass, 0)?;

                let type_id = var
                    .result_type
                    .ok_or_else(|| ReflectError::VariableWithoutReturnType(var.clone()))?;
                let mut descriptor_info =
                    self.get_descriptor_type_for_var(type_id, storage_class)?;

                if let Some(name) = names.get(&var_id) {
                    // TODO: Might do this way earlier
                    if name == "$Globals" {
                        return Err(ReflectError::BindingGlobalParameterBuffer);
                    }

                    descriptor_info.name = name.to_owned();
                }

                let inserted = current_set.insert(binding, descriptor_info);
                assert!(
                    inserted.is_none(),
                    "Can't bind to the same slot twice within the same shader"
                );
            }
        }
        Ok(unique_sets)
    }

    /// Returns the byte offset to the last variable in a struct
    /// used to calculate the size of the struct
    fn byte_offset_to_last_var(
        reflect: &Module,
        struct_instruction: &Instruction,
    ) -> Result<u32, ReflectError> {
        debug_assert!(struct_instruction.class.opcode == spirv::Op::TypeStruct);

        // if there are less then two members there is no offset to use, early out
        if struct_instruction.operands.len() < 2 {
            return Ok(0);
        }

        let result_id = struct_instruction
            .result_id
            .ok_or_else(|| ReflectError::MissingResultId(struct_instruction.clone()))?;

        // return the highest offset value
        Ok(filter_annotations_with_id(&reflect.annotations, result_id)?
            .iter()
            .filter(|i| i.class.opcode == spirv::Op::MemberDecorate)
            .filter_map(|&i| match get_operand_at!(i, Operand::Decoration, 2) {
                Ok(decoration) if decoration == spirv::Decoration::Offset => {
                    Some(get_operand_at!(i, Operand::LiteralBit32, 3))
                }
                Err(err) => Some(Err(err)),
                _ => None,
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .max()
            .unwrap_or(0))
    }

    fn calculate_variable_size_bytes(
        reflect: &Module,
        type_instruction: &Instruction,
    ) -> Result<u32, ReflectError> {
        match type_instruction.class.opcode {
            spirv::Op::TypeInt | spirv::Op::TypeFloat => {
                debug_assert!(!type_instruction.operands.is_empty());
                Ok(get_operand_at!(type_instruction, Operand::LiteralBit32, 0)? / 8)
            }
            spirv::Op::TypeVector | spirv::Op::TypeMatrix => {
                debug_assert!(type_instruction.operands.len() == 2);
                let type_id = get_operand_at!(type_instruction, Operand::IdRef, 0)?;
                let var_type_instruction =
                    find_instructions_assigning_to_id(&reflect.types_global_values, type_id)?;
                let type_size_bytes =
                    Self::calculate_variable_size_bytes(reflect, var_type_instruction)?;

                let type_constant_count =
                    get_operand_at!(type_instruction, Operand::LiteralBit32, 1)?;
                Ok(type_size_bytes * type_constant_count)
            }
            spirv::Op::TypeArray => {
                debug_assert!(type_instruction.operands.len() == 2);
                let type_id = get_operand_at!(type_instruction, Operand::IdRef, 0)?;
                let var_type_instruction =
                    find_instructions_assigning_to_id(&reflect.types_global_values, type_id)?;
                let type_size_bytes =
                    Self::calculate_variable_size_bytes(reflect, var_type_instruction)?;

                let var_constant_id = get_operand_at!(type_instruction, Operand::IdRef, 1)?;
                let constant_instruction = find_instructions_assigning_to_id(
                    &reflect.types_global_values,
                    var_constant_id,
                )?;
                let type_constant_count =
                    get_operand_at!(constant_instruction, Operand::LiteralBit32, 0)?;

                Ok(type_size_bytes * type_constant_count)
            }
            spirv::Op::TypeStruct => {
                if !type_instruction.operands.is_empty() {
                    let byte_offset = Self::byte_offset_to_last_var(reflect, type_instruction)?;
                    let last_var_idx = type_instruction.operands.len() - 1;
                    let id_ref = get_operand_at!(type_instruction, Operand::IdRef, last_var_idx)?;
                    let type_instruction =
                        find_instructions_assigning_to_id(&reflect.types_global_values, id_ref)?;
                    Ok(byte_offset
                        + Self::calculate_variable_size_bytes(reflect, type_instruction)?)
                } else {
                    Ok(0)
                }
            }
            spirv::Op::TypePointer => {
                let memory_model = reflect
                    .memory_model
                    .as_ref()
                    .ok_or(ReflectError::MissingMemoryModel)?;
                let addressing_model = get_operand_at!(memory_model, Operand::AddressingModel, 0)?;

                let storage_class = get_operand_at!(type_instruction, Operand::StorageClass, 0)?;

                // https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html#Addressing_Model
                // https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html#Storage_Class
                match (addressing_model, storage_class) {
                    (
                        // https://github.com/KhronosGroup/SPIRV-Registry/blob/main/extensions/KHR/SPV_KHR_physical_storage_buffer.asciidoc
                        spirv::AddressingModel::PhysicalStorageBuffer64,
                        spirv::StorageClass::PhysicalStorageBuffer,
                    ) => Ok(8),
                    (a, s) => Err(ReflectError::InvalidAddressingModelAndStorageClass(a, s)),
                }
            }
            x => panic!("Size computation for {:?} unsupported", x),
        }
    }

    pub fn get_push_constant_range(&self) -> Result<Option<PushConstantInfo>, ReflectError> {
        let reflect = &self.0;

        let push_constants = reflect
            .types_global_values
            .iter()
            .filter(|i| i.class.opcode == spirv::Op::Variable)
            .filter_map(|i| {
                let cls = get_operand_at!(*i, Operand::StorageClass, 0);
                match cls {
                    Ok(cls) if cls == spirv::StorageClass::PushConstant => Some(Ok(i)),
                    Err(err) => Some(Err(err)),
                    _ => None,
                }
            })
            .collect::<Result<Vec<_>>>()?;

        if push_constants.len() > 1 {
            return Err(ReflectError::TooManyPushConstants);
        }

        let push_constant = match push_constants.into_iter().next() {
            Some(push_constant) => push_constant,
            None => return Ok(None),
        };

        let instruction = find_instructions_assigning_to_id(
            &reflect.types_global_values,
            push_constant.result_type.unwrap(),
        )?;

        // resolve type if the type instruction is a pointer
        let instruction = if instruction.class.opcode == spirv::Op::TypePointer {
            let ptr_storage_class = get_operand_at!(instruction, Operand::StorageClass, 0)?;
            assert_eq!(spirv::StorageClass::PushConstant, ptr_storage_class);
            let element_type_id = get_operand_at!(instruction, Operand::IdRef, 1)?;
            find_instructions_assigning_to_id(&reflect.types_global_values, element_type_id)?
        } else {
            instruction
        };

        let size_bytes = Self::calculate_variable_size_bytes(reflect, instruction)?;

        Ok(Some(PushConstantInfo {
            size: size_bytes,
            offset: 0,
        }))
    }

    pub fn disassemble(&self) -> String {
        use rspirv::binary::Disassemble;
        self.0.disassemble()
    }
}
pub fn filter_instructions_that_have_id(instr: &[Instruction]) -> Vec<&Instruction> {
    instr
        .iter()
        .filter_map(|a| {
            let op = get_operand_at!(a, Operand::IdRef, 0);
            match op {
                Ok(_) => Some(a),
                Err(_) => None,
            }
        })
        .collect::<Vec<_>>()
}

/// Returns the first `Instruction` assigning to `id` (ie. `result_id == Some(id)`)
pub fn find_instructions_assigning_to_id(instr: &[Instruction], id: Id) -> Result<&Instruction> {
    instr
        .iter()
        .find(|instr| instr.result_id == Some(id))
        .ok_or(ReflectError::UnassignedResultId(id))
}
/// Returns all "decorators" where the first operand (`Instruction::operands[0]`) equals `IdRef(id)`
/// Gives `Err` if instruction doesn't have `IdRef` at `operand[0]`
pub fn filter_annotations_with_id(anno: &[Instruction], id: Id) -> Result<Vec<&Instruction>> {
    anno.iter()
        .filter_map(|a| {
            let op = get_operand_at!(a, Operand::IdRef, 0);
            match op {
                Ok(idref) if idref == id => Some(Ok(a)),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        })
        .collect::<Result<Vec<_>>>()
}
