use super::raw::*;
use std::ffi::CStr;
#[derive(
    Debug,
    derive_more::From,
    derive_more::Into,
    derive_more::AsRef,
    derive_more::AsMut,
    PartialEq,
    Clone,
)]
pub struct ExtensionName(pub(super) [i8; ExtensionName::MAX_SIZE]);
impl ExtensionName {
    pub const MAX_SIZE: usize = ash::vk::MAX_EXTENSION_NAME_SIZE;
    pub const fn from_cstr(cstr: &CStr) -> Self {
        Self(from_cstr_to_array(cstr))
    }
}
impl From<&CStr> for ExtensionName {
    fn from(value: &CStr) -> Self {
        Self(from_cstr_to_array(value))
    }
}

impl std::ops::Deref for ExtensionName {
    type Target = CStr;
    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.0.as_ptr()) }
    }
}
#[derive(
    Debug,
    derive_more::From,
    derive_more::Into,
    derive_more::AsRef,
    derive_more::AsMut,
    PartialEq,
    Clone,
)]
pub struct Description(pub(super) [i8; Description::MAX_SIZE]);
impl Description {
    pub const MAX_SIZE: usize = ash::vk::MAX_DESCRIPTION_SIZE;
    pub const fn from_cstr(cstr: &CStr) -> Self {
        Self(from_cstr_to_array(cstr))
    }
}

impl std::ops::Deref for Description {
    type Target = CStr;
    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.0.as_ptr()) }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub name: ExtensionName,
    pub spec_version: u32,
    pub implementation_version: u32,
    pub description: Description,
}
impl Layer {
    pub const VALIDATIONLAYER: ExtensionName =
        ExtensionName::from_cstr(c"VK_LAYER_KHRONOS_validation");
}

impl From<ash::vk::LayerProperties> for Layer {
    fn from(prop: ash::vk::LayerProperties) -> Self {
        Self {
            name: ExtensionName(prop.layer_name),
            spec_version: prop.spec_version,
            implementation_version: prop.implementation_version,
            description: Description(prop.description),
        }
    }
}

#[derive(Debug, derive_more::From, derive_more::Into, PartialEq)]
pub struct ExtensionProperties {
    pub name: ExtensionName,
    pub spec_version: u32,
}

impl From<ash::vk::ExtensionProperties> for ExtensionProperties {
    fn from(prop: ash::vk::ExtensionProperties) -> Self {
        Self {
            name: ExtensionName(prop.extension_name),
            spec_version: prop.spec_version,
        }
    }
}

// Due to the many linking chains, this type simply outputs the handle when printed using Debug.
#[derive(
    derive_more::From,
    derive_more::Deref,
    derive_more::DerefMut,
    derive_more::AsRef,
    derive_more::AsMut,
)]
pub struct HandleWrapper<T: ash::vk::Handle>(T);

impl<T: ash::vk::Handle> std::fmt::Debug for HandleWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle: {:?}", <T as ash::vk::Handle>::TYPE)
    }
}
