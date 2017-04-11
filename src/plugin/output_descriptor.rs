use std::ffi::CString;
use std::ffi::CStr;
use std::os::raw::c_char;
use ::cxx_util::{CxxInnerVector,CxxVector,CxxString};
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
    pub quantize_step: Option<f32>,
    pub sample_type: SampleType,
    pub has_duration: bool,
}

impl OutputDescriptor {
    pub fn from(ptr: *const CxxOutputDescriptor) -> Self {
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
        let has_fixed_bin_count = unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> bool as "bool" {
            return ptr->hasFixedBinCount;
        })};
        let bin_count = match has_fixed_bin_count {
            true => {
                Some(unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> usize as "size_t" {
                    return ptr->binCount;
                })})
            }
            false => None
        };
        let bin_names = match has_fixed_bin_count {
            true => {
                let cxx_vec: CxxVector<CxxString> = unsafe { CxxVector::from(cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> *mut CxxInnerVector as "const std::vector<std::string>*" {
                    return &(ptr->binNames);
                }))};
                let vec = cxx_vec.to_vec();
                cxx_vec.into_raw();
                Some(vec)
                //Some(unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> usize as "size_t" {
                //    return ptr->binCount;
                //})})
            }
            false => None
        };
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
        };
        let max_value = match has_known_extents {
            true => {
                Some(unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> f32 as "float" {
                    return ptr->maxValue;
                })})
            }
            false => None
        };
        let extents = match (min_value,max_value) {
            (Some(min),Some(max)) => Some((min,max)),
            _ => None
        };
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
        let sample_type = match unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> i32 as "int" {
                auto en = ptr->sampleType; // MEMS
                switch (en) {
                    case Vamp::Plugin::OutputDescriptor::SampleType::OneSamplePerStep:
                        return 0;
                    case Vamp::Plugin::OutputDescriptor::SampleType::FixedSampleRate:
                        return 1;
                    case Vamp::Plugin::OutputDescriptor::SampleType::VariableSampleRate:
                        return 2;
                }
            })} {
            0 => SampleType::OneSamplePerStep,
            m if ((m == 2)||(m == 1)) => {
                let sample_rate = unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> f32 as "float" {
                    return ptr->sampleRate;
                })};
                match m {
                    1 => SampleType::FixedSampleRate(sample_rate),
                    2 => SampleType::VariableSampleRate(sample_rate),
                    _ => unreachable!()
                }
            }
            _ => unreachable!()
        };
        let has_duration = unsafe { cpp!([ptr as "Vamp::Plugin::OutputDescriptor*"] -> bool as "bool" {
            return ptr->hasDuration;
        })};
        return OutputDescriptor {
            identifier: identifier,
            name: name,
            description: description,
            unit: unit,
            bin_count: bin_count,
            bin_names: bin_names,
            extents: extents,
            quantize_step: quantize_step,
            sample_type: sample_type,
            has_duration: has_duration
        }
    }
}
