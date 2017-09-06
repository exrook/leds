use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use cxx_util::{CxxInnerVector, CxxVector, CxxString};

pub enum CxxParameterDescriptor {}
#[derive(Debug)]
pub struct ParameterDescriptor {
    /// The name of the parameter, in computer-usable form.
    ///
    /// Should be reasonably short, and may only contain the characters [a-zA-Z0-9_-].
    pub identifier: CString,
    /// The human-readable name of the parameter.
    pub name: CString,
    /// A human-readable short text describing the parameter.
    ///
    /// May be empty if the name has said it all already.
    pub description: CString,
    /// The unit of the parameter, in human-readable form.
    pub unit: CString,
    /// The minimum value of the parameter.
    pub min_value: f32,
    /// The maximum value of the parameter.
    pub max_value: f32,
    /// The default value of the parameter.
    ///
    /// The plugin should ensure that parameters have this value on initialisation (i.e. the
    /// host is not required to explicitly set parameters if it wants to use their default values).
    pub default_value: f32,
    /// Quantization resolution of the parameter values (e.g. 1.0 if they are all integers), if
    /// present.
    pub quantize_step: Option<f32>,
    /// Names for the quantized values.
    ///
    /// If quantize_step is present, this may either be empty or contain one string for each of the quantize steps from minValue up to maxValue inclusive.
    ///
    /// If these names are provided, they should be shown to the user in preference to the values themselves. The user may never see the actual numeric values unless they are also encoded in the names.
    pub value_names: Option<Vec<CString>>,
}

impl CxxParameterDescriptor {
    pub fn to_rust(&self) -> ParameterDescriptor {
        ParameterDescriptor::from(self)
    }
}

impl ParameterDescriptor {
    /// Create a rust ParameterDescriptor object from a C++ reference
    pub fn from(ptr: *const CxxParameterDescriptor) -> Self {
        let identifier = unsafe { CStr::from_ptr(cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> *const c_char as "const char*" {
            return ptr->identifier.c_str();
        }))}.to_owned();
        let name = unsafe { CStr::from_ptr(cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> *const c_char as "const char*" {
            return ptr->name.c_str();
        }))}.to_owned();
        let description = unsafe { CStr::from_ptr(cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> *const c_char as "const char*" {
            return ptr->description.c_str();
        }))}.to_owned();
        let unit = unsafe { CStr::from_ptr(cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> *const c_char as "const char*" {
            return ptr->unit.c_str();
        }))}.to_owned();
        let min_value = unsafe {
            cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> f32 as "float" {
            return ptr->minValue;
        })
        };
        let max_value = unsafe {
            cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> f32 as "float" {
            return ptr->maxValue;
        })
        };
        let default_value = unsafe {
            cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> f32 as "float" {
            return ptr->defaultValue;
        })
        };
        let is_quantized = unsafe {
            cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> bool as "bool" {
            return ptr->isQuantized;
        })
        };
        let quantize_step = match is_quantized {
            true => {
                Some(unsafe {
                    cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> f32 as "float" {
                    return ptr->quantizeStep;
                })
                })
            }
            false => None,
        };
        let value_names = match is_quantized {
            true => {
                let tmp: CxxVector<CxxString> = unsafe {
                    CxxVector::from(
                        cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> *mut CxxInnerVector as "const std::vector<std::string>*" {
                    return &(ptr->valueNames);
                }),
                    )
                };
                let v = tmp.to_vec();
                tmp.into_raw();
                Some(v)
            }
            false => None,
        };
        return ParameterDescriptor {
            identifier: identifier,
            name: name,
            description: description,
            unit: unit,
            min_value: min_value,
            max_value: max_value,
            default_value: default_value,
            quantize_step: quantize_step,
            value_names: value_names,
        };
    }
}
