//! This module contains the types used in the SPIR-V standard.

#![allow(dead_code)]

use std::rc::Rc;

use rspirv::spirv::StorageClass;

const BYTE_SIZE: u16 = 8;
pub trait NumType: MinSize + Alignment {}
// Minimum required bytes to represent the type as bits.
// This does not account for necessary padding.
pub trait MinSize {
    fn min_bits_size(&self) -> u16;
    fn min_bytes_size(&self) -> u16 {
        let bits = self.min_bits_size();
        (bits + (BYTE_SIZE - bits % BYTE_SIZE) % BYTE_SIZE) / BYTE_SIZE
    }
}
/// As defined by the standard
pub trait Alignment {
    fn scalar_align(&self) -> u16;
    fn base_align(&self) -> u16;
    fn extended_align(&self) -> u16;
    fn get_alignment(&self, usage: impl Into<AlignmentUsage>) -> u16 {
        let usage = usage.into();
        match usage {
            AlignmentUsage::Scalar => self.scalar_align(),
            AlignmentUsage::Base => self.base_align(),
            AlignmentUsage::Extended => self.extended_align(),
        }
    }
}

pub enum AlignmentUsage {
    Scalar,
    Base,
    Extended,
}

// This exists because vector sizes are limited to 1<=len<=4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Size1,
    Size2,
    Size3,
    Size4,
}

impl From<u16> for Size {
    fn from(n: u16) -> Self {
        match n {
            1 => Self::Size1,
            2 => Self::Size2,
            3 => Self::Size3,
            4 => Self::Size4,
            _ => unreachable!(),
        }
    }
}

impl From<Size> for u16 {
    fn from(val: Size) -> Self {
        match val {
            Size::Size1 => 1,
            Size::Size2 => 2,
            Size::Size3 => 3,
            Size::Size4 => 4,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Type {
    Int(Int),
    Float(Float),
    Vector(Vector),
    Mat(Mat),
    Array(Array),
    RunTimeArray(RunTimeArray),
    Struct(Struct),
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(arg0) => arg0.fmt(f),
            Self::Float(arg0) => arg0.fmt(f),
            Self::Vector(arg0) => arg0.fmt(f),
            Self::Mat(arg0) => arg0.fmt(f),
            Self::Array(arg0) => arg0.fmt(f),
            Self::RunTimeArray(arg0) => arg0.fmt(f),
            Self::Struct(arg0) => arg0.fmt(f),
        }
    }
}

impl MinSize for Type {
    fn min_bits_size(&self) -> u16 {
        match self {
            Self::Int(x) => x.min_bits_size(),
            Self::Float(x) => x.min_bits_size(),
            Self::Vector(x) => x.min_bits_size(),
            Self::Mat(x) => x.min_bits_size(),
            Self::Array(x) => x.min_bits_size(),
            Self::RunTimeArray(_) => 0,
            Self::Struct(x) => x.min_bits_size(),
        }
    }
}

impl Alignment for Type {
    fn base_align(&self) -> u16 {
        match self {
            Self::Int(x) => x.base_align(),
            Self::Float(x) => x.base_align(),
            Self::Vector(x) => x.base_align(),
            Self::Mat(x) => x.base_align(),
            Self::Array(x) => x.base_align(),
            Self::RunTimeArray(x) => x.base_align(),
            Self::Struct(x) => x.base_align(),
        }
    }

    fn extended_align(&self) -> u16 {
        match self {
            Type::Int(x) => x.extended_align(),
            Type::Float(x) => x.extended_align(),
            Type::Vector(x) => x.extended_align(),
            Type::Mat(x) => x.extended_align(),
            Type::Array(x) => x.extended_align(),
            Type::RunTimeArray(x) => x.extended_align(),
            Type::Struct(x) => x.extended_align(),
        }
    }

    fn scalar_align(&self) -> u16 {
        match self {
            Type::Int(x) => x.scalar_align(),
            Type::Float(x) => x.scalar_align(),
            Type::Vector(x) => x.scalar_align(),
            Type::Mat(x) => x.scalar_align(),
            Type::Array(x) => x.scalar_align(),
            Type::RunTimeArray(x) => x.scalar_align(),
            Type::Struct(x) => x.scalar_align(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vector {
    pub inner_type: Rc<Type>,
    pub size: Size,
}
#[derive(Debug, Clone, PartialEq, Eq)]

pub struct Int {
    pub bits: u16,
    pub issigned: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Float {
    pub bits: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mat {
    pub size: Size,
    pub inner_type: Rc<Type>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Array {
    pub inner_type: Rc<Type>,
    pub len: u16,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunTimeArray {
    pub inner_type: Rc<Type>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    pub fields: Vec<Rc<Type>>,
    /// An empty structure has a base alignment equal to the size of the smallest scalar type permitted by the capabilities declared in the SPIR-V module. (e.g., for a 1 byte aligned empty struct in the StorageBuffer storage class, StorageBuffer8BitAccess or UniformAndStorageBuffer8BitAccess must be declared in the SPIR-V module.)
    pub(super) base_alignment: u16,
    pub(super) block_decor: bool,
}

impl MinSize for Int {
    fn min_bits_size(&self) -> u16 {
        self.bits
    }
}
impl Alignment for Int {
    fn scalar_align(&self) -> u16 {
        let residue = if self.bits % BYTE_SIZE != 0 { 1 } else { 0 };
        self.bits / BYTE_SIZE + residue
    }
    fn base_align(&self) -> u16 {
        self.scalar_align()
    }
    fn extended_align(&self) -> u16 {
        self.scalar_align()
    }
}
impl MinSize for Float {
    fn min_bits_size(&self) -> u16 {
        self.bits
    }
}

impl Alignment for Float {
    fn scalar_align(&self) -> u16 {
        let residue = if self.bits % BYTE_SIZE != 0 { 1 } else { 0 };
        self.bits / BYTE_SIZE + residue
    }
    fn base_align(&self) -> u16 {
        self.scalar_align()
    }
    fn extended_align(&self) -> u16 {
        self.scalar_align()
    }
}

impl MinSize for Vector {
    fn min_bits_size(&self) -> u16 {
        self.inner_type.min_bits_size()
    }
}
impl Alignment for Vector {
    fn scalar_align(&self) -> u16 {
        self.inner_type.scalar_align()
    }
    fn base_align(&self) -> u16 {
        match self.size {
            Size::Size2 => self.inner_type.base_align() * 2,
            Size::Size3 => self.inner_type.base_align() * 4,
            Size::Size4 => self.inner_type.base_align() * 4,
            _ => unreachable!(),
        }
    }
    fn extended_align(&self) -> u16 {
        self.base_align()
    }
}

impl Array {
    /// Matrixes use the same alligment as Array, so they are defined here once.
    fn inner_scalar_align(inner_type: &Rc<Type>) -> u16 {
        inner_type.scalar_align()
    }
    /// Matrixes use the same alligment as Array, so they are defined here once.
    fn inner_base_align(inner_type: &Rc<Type>) -> u16 {
        inner_type.base_align()
    }
    /// Matrixes use the same alligment as Array, so they are defined here once.
    fn inner_extended_align(inner_type: &Rc<Type>) -> u16 {
        let n = inner_type.extended_align();
        //Round up to nearest 16
        n + (16 - n % 16) % 16
    }
}

impl Alignment for RunTimeArray {
    fn scalar_align(&self) -> u16 {
        Array::inner_scalar_align(&self.inner_type)
    }
    fn base_align(&self) -> u16 {
        Array::inner_base_align(&self.inner_type)
    }
    fn extended_align(&self) -> u16 {
        Array::inner_extended_align(&self.inner_type)
    }
}
impl MinSize for Array {
    fn min_bits_size(&self) -> u16 {
        self.inner_type.min_bits_size() * self.len
    }
}
impl Alignment for Array {
    fn scalar_align(&self) -> u16 {
        Array::inner_scalar_align(&self.inner_type)
    }
    fn base_align(&self) -> u16 {
        Array::inner_base_align(&self.inner_type)
    }
    fn extended_align(&self) -> u16 {
        Array::inner_extended_align(&self.inner_type)
    }
}
impl MinSize for Mat {
    fn min_bits_size(&self) -> u16 {
        self.inner_type.min_bits_size()
            * match self.size {
                Size::Size1 => 1,
                Size::Size2 => 2 * 2,
                Size::Size3 => 3 * 3,
                Size::Size4 => 4 * 4,
            }
    }
}
impl Alignment for Mat {
    fn scalar_align(&self) -> u16 {
        // Inherits the same array alignment.
        Array::inner_scalar_align(&self.inner_type)
    }

    fn base_align(&self) -> u16 {
        // Inherits the same array alignment.
        Array::inner_base_align(&self.inner_type)
    }

    fn extended_align(&self) -> u16 {
        // Inherits the same array alignment.
        Array::inner_extended_align(&self.inner_type)
    }
}
impl MinSize for Struct {
    fn min_bits_size(&self) -> u16 {
        self.fields.iter().map(|x| x.min_bits_size()).sum()
    }
}
impl Alignment for Struct {
    fn scalar_align(&self) -> u16 {
        self.fields
            .iter()
            .map(|x| x.scalar_align())
            .max()
            .unwrap_or(0)
    }

    fn base_align(&self) -> u16 {
        self.fields
            .iter()
            .map(|x| x.base_align())
            .max()
            .unwrap_or(self.base_alignment)
    }

    fn extended_align(&self) -> u16 {
        let n = self
            .fields
            .iter()
            .map(|x| x.extended_align())
            .max()
            .unwrap();
        //Round up to nearest 16
        n + (16 - n % 16) % 16
    }
}

/*
Every member of an OpTypeStruct that is required to be explicitly laid out must be aligned according to the first matching rule as follows. If the struct is contained in pointer types of multiple storage classes, it must satisfy the requirements for every storage class used to reference it.

    If the scalarBlockLayout feature is enabled on the device and the storage class is Uniform, StorageBuffer, PhysicalStorageBuffer, ShaderRecordBufferKHR, or PushConstant then every member must be aligned according to its scalar alignment.

    If the workgroupMemoryExplicitLayoutScalarBlockLayout feature is enabled on the device and the storage class is Workgroup then every member must be aligned according to its scalar alignment.

    All vectors must be aligned according to their scalar alignment.

    If the uniformBufferStandardLayout feature is not enabled on the device, then any member of an OpTypeStruct with a storage class of Uniform and a decoration of Block must be aligned according to its extended alignment.

    Every other member must be aligned according to its base alignment.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceExtensions {
    scalar_block_layout: bool,
    workgroup_memory_explicit_layout_scalar_block_layout: bool,
    uniform_buffer_standard_layout: bool,
}
impl Default for DeviceExtensions {
    fn default() -> Self {
        Self {
            scalar_block_layout: false,
            workgroup_memory_explicit_layout_scalar_block_layout: false,
            uniform_buffer_standard_layout: true,
        }
    }
}

impl Struct {
    ///
    pub fn get_alignment_type_for_fields(
        &self,
        storage_class: &StorageClass,
        ext: &DeviceExtensions,
    ) -> Vec<AlignmentUsage> {
        self.fields
            .iter()
            .map(|x| match (x.as_ref(), ext, storage_class) {
                (
                    _,
                    DeviceExtensions {
                        scalar_block_layout: true,
                        ..
                    },
                    StorageClass::Uniform
                    | StorageClass::StorageBuffer
                    | StorageClass::PhysicalStorageBuffer
                    | &StorageClass::ShaderRecordBufferKHR
                    | StorageClass::PushConstant,
                ) => AlignmentUsage::Scalar,
                (
                    _,
                    DeviceExtensions {
                        workgroup_memory_explicit_layout_scalar_block_layout: true,
                        ..
                    },
                    StorageClass::Workgroup,
                ) => AlignmentUsage::Scalar,
                (Type::Vector(_), _, _) => AlignmentUsage::Scalar,
                (
                    Type::Struct(Struct {
                        block_decor: true, ..
                    }),
                    DeviceExtensions {
                        uniform_buffer_standard_layout: false,
                        ..
                    },
                    StorageClass::Uniform,
                ) => AlignmentUsage::Extended,
                _ => AlignmentUsage::Base,
            })
            .collect()
    }
    /// These offsets are calculated from the start of the struct.
    pub fn get_fields_offset(
        &self,
        ext: &DeviceExtensions,
        storage_class: &StorageClass,
    ) -> Vec<u16> {
        self.get_alignment_type_for_fields(storage_class, ext)
            .into_iter()
            .zip(self.fields.iter().map(|x| (x, x.min_bytes_size())))
            .scan(0u16, |acc, (align_type, (field, size))| {
                let align = field.get_alignment(align_type);
                let realign = (align - *acc % align) % align;
                let pos = *acc + realign;
                *acc += pos + size;
                Some(pos)
            })
            .collect()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum VectorStraddle {
    #[error("The vector straddles two 16 byte blocks.")]
    OccupiesTwo16ByteBlocks,
    #[error("The vector is not aligned to a 16 byte block.")]
    NotAlignedTo16ByteBlock,
}
/// A member is defined to improperly straddle if either of the following are true:
/// It is a vector with total size less than or equal to 16 bytes, and has Offset decorations placing its first byte at F and its last byte at L, where floor(F / 16) != floor(L / 16).
/// It is a vector with total size greater than 16 bytes and has its Offset decorations placing its first byte at a non-integer multiple of 16.
/// `offset` is the offset the vector is placed inside the struct (or it can be relative to the
/// whole memory block.)
fn is_vector_straddling_inside_struct_aligned(
    vector: &Vector,
    offset: u32,
) -> Result<(), VectorStraddle> {
    let size = vector.inner_type.min_bytes_size();
    let size = size as u32;
    let first_byte = offset;
    let last_byte = offset + size;
    if size < 16 {
        let f = first_byte / 16;
        let l = last_byte / 16;
        (f != l)
            .then(|| ())
            .ok_or(VectorStraddle::OccupiesTwo16ByteBlocks)
    } else {
        (first_byte % 16 != 0)
            .then(|| ())
            .ok_or(VectorStraddle::NotAlignedTo16ByteBlock)
    }
}

/*



The scalar alignment of the type of an OpTypeStruct member is defined recursively as follows:
    A scalar of size N has a scalar alignment of N.
    A vector type has a scalar alignment equal to that of its component type.
    An array type has a scalar alignment equal to that of its element type.
    A structure has a scalar alignment equal to the largest scalar alignment of any of its members.
    A matrix type inherits scalar alignment from the equivalent array declaration.
The base alignment of the type of an OpTypeStruct member is defined recursively as follows:
    A scalar has a base alignment equal to its scalar alignment.
    A two-component vector has a base alignment equal to twice its scalar alignment.
    A three- or four-component vector has a base alignment equal to four times its scalar alignment.
    An array has a base alignment equal to the base alignment of its element type.
    A structure has a base alignment equal to the largest base alignment of any of its members. An empty structure has a base alignment equal to the size of the smallest scalar type permitted by the capabilities declared in the SPIR-V module. (e.g., for a 1 byte aligned empty struct in the StorageBuffer storage class, StorageBuffer8BitAccess or UniformAndStorageBuffer8BitAccess must be declared in the SPIR-V module.)
    A matrix type inherits base alignment from the equivalent array declaration.

The extended alignment of the type of an OpTypeStruct member is similarly defined as follows:
    A scalar or vector type has an extended alignment equal to its base alignment.
    An array or structure type has an extended alignment equal to the largest extended alignment of any of its members, rounded up to a multiple of 16.
    A matrix type inherits extended alignment from the equivalent array declaration.



Standard Buffer Layout
Every member of an OpTypeStruct that is required to be explicitly laid out must be aligned according to the first matching rule as follows. If the struct is contained in pointer types of multiple storage classes, it must satisfy the requirements for every storage class used to reference it.
    If the scalarBlockLayout feature is enabled on the device and the storage class is Uniform, StorageBuffer, PhysicalStorageBuffer, ShaderRecordBufferKHR, or PushConstant then every member must be aligned according to its scalar alignment.
    If the workgroupMemoryExplicitLayoutScalarBlockLayout feature is enabled on the device and the storage class is Workgroup then every member must be aligned according to its scalar alignment.
    All vectors must be aligned according to their scalar alignment.
    If the uniformBufferStandardLayout feature is not enabled on the device, then any member of an OpTypeStruct with a storage class of Uniform and a decoration of Block must be aligned according to its extended alignment.
    Every other member must be aligned according to its base alignment.

NOTE:
Even if scalar alignment is supported, it is generally more performant to use the base alignment.
The memory layout must obey the following rules:
    The Offset decoration of any member must be a multiple of its alignment.
    Any ArrayStride or MatrixStride decoration must be a multiple of the alignment of the array or matrix as defined above.
If one of the conditions below applies
    The storage class is Uniform, StorageBuffer, PhysicalStorageBuffer, ShaderRecordBufferKHR, or PushConstant, and the scalarBlockLayout feature is not enabled on the device.
    The storage class is Workgroup, and either the struct member is not part of a Block or the workgroupMemoryExplicitLayoutScalarBlockLayout feature is not enabled on the device.
    The storage class is any other storage class.
then memory layout must also obey the following rules:
    Vectors must not improperly straddle, as defined above.
    The Offset decoration of a member must not place it between the end of a structure, an array or a matrix and the next multiple of the alignment of that structure, array or matrix.


NOTE:
The std430 layout in GLSL satisfies these rules for types using the base alignment. The std140 layout satisfies the rules for types using the extended alignment.
*/
