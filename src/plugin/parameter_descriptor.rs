use std::ffi::{CStr,CString};
use std::os::raw::c_char;
use ::cxx_util::{CxxInnerVector,CxxVector};

pub enum CxxParameterDescriptor {}
pub struct ParameterDescriptor {
    identifier: CString,
    name: CString,
    description: CString,
    unit: CString,
    min_value: f32,
    max_value: f32,
    default_value: f32,
    quantize_step: Option<f32>,
    value_names: Option<Vec<CString>>,
}

impl ParameterDescriptor { 
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
        let min_value = unsafe { cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> f32 as "float" {
            return ptr->minValue;
        })};
        let max_value = unsafe { cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> f32 as "float" {
            return ptr->maxValue;
        })};
        let default_value = unsafe { cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> f32 as "float" {
            return ptr->defaultValue;
        })};
        let is_quantized = unsafe { cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> bool as "bool" {
            return ptr->isQuantized;
        })};
        let quantize_step = match is_quantized {
            true => {
                Some(unsafe { cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> f32 as "float" {
                    return ptr->quantizeStep;
                })})
            }
            false => {
                None
            }
        };
        let value_names = match is_quantized {
            true => {
                let tmp = unsafe { CxxVector::from(cpp!([ptr as "Vamp::PluginBase::ParameterDescriptor*"] -> *mut CxxInnerVector as "const std::vector<std::string>*" {
                    return &(ptr->valueNames);
                }))};
                Some(tmp.to_vec())
            }
            false => {
                None
            }
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
        }
    }
}
