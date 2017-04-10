use std::ffi::CString;
pub enum SampleType {
    OneSamplePerStep,
    FixedSampleRate(f32),
    VariableSampleRate(f32)
}
pub enum CxxOutputDescriptor {}
pub struct OutputDescriptor {
    pub identifier: CString,
    pub name: CString,
    pub description: CString,
    pub unit: CString,
    /// Present if there is a fixed bin size, if zero, output is point data
    pub bin_count: Option<usize>,
    pub bin_names: Option<Vec<CString>>,
    /// (Min,Max) possible range of values if present
    pub extents: Option<(f32,f32)>,
    /// If present, resolution values are quantized to
    pub quantizeStep: Option<f32>,
    pub sample_type: SampleType,
    pub has_duration: bool,
}

impl CxxOutputDescriptor {
    pub fn to_output_descriptor(&self) -> OutputDescriptor {
        let identifier = unsafe { CStr::from_ptr(cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> *const c_char as "const char*" {
            return ptr->identifier.c_str();
        }))}.to_owned();
        let name = unsafe { CStr::from_ptr(cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> *const c_char as "const char*" {
            return ptr->name.c_str();
        }))}.to_owned();
        let description = unsafe { CStr::from_ptr(cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> *const c_char as "const char*" {
            return ptr->description.c_str();
        }))}.to_owned();
        let unit = unsafe { CStr::from_ptr(cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> *const c_char as "const char*" {
            return ptr->unit.c_str();
        }))}.to_owned();
        let has_fixed_bin_count = unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> f32 as "float" {
            return ptr->hasFixedBinCount;
        })};
        let bin_count = match has_fixed_bin_count {
            true => {
                Some(unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> usize as "size_t" {
                    return ptr->binCount;
                })})
            }
            false => None
        }
        let bin_names = match has_fixed_bin_count {
            true => {
                unimplemented!();
                Some(unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> usize as "size_t" {
                    return ptr->binCount;
                })})
            }
            false => None
        }
        let has_known_extents = unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> bool as "bool" {
            return ptr->hasKnownExtents;
        })};
        let min_value = match has_known_extents {
            true => {
                Some(unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> f32 as "float" {
                    return ptr->minValue;
                })})
            }
            false => None
        }
        let max_value = match has_known_extents {
            true => {
                Some(unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> f32 as "float" {
                    return ptr->maxValue;
                })})
            }
            false => None
        }
        let is_quantized = unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> bool as "bool" {
            return ptr->isQuantized;
        })};
        let quantize_step = match is_quantized {
            true => {
                Some(unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> f32 as "float" {
                    return ptr->quantizeStep;
                })})
            }
            false => {
                None
            }
        };
        let value_names = match is_quantized {
            true => {
                let tmp = unsafe { CxxVector::from(cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> *mut CxxInnerVector as "const std::vector<std::string>*" {
                    return &(ptr->valueNames);
                }))};
                Some(tmp.to_vec())
            }
            false => {
                None
            }
        };
        let is_quantized = unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> bool as "bool" {
            return ptr->isQuantized;
        })};
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
        unimplemented!();
    }
}
