use std::ffi::CString;
use std::ffi::CStr;
use std::os::raw::c_char;
use ::cxx_util::{CxxInnerVector,CxxVector,CxxString};
#[derive(Debug)]
pub enum SampleType {
    OneSamplePerStep,
    FixedSampleRate(f32),
    VariableSampleRate(f32)
}
pub enum CxxOutputDescriptor {}
#[derive(Debug)]
pub struct OutputDescriptor {
    /// The name of the output, in computer-usable form.
    /// 
    /// Should be reasonably short and without whitespace or punctuation, using the characters [a-zA-Z0-9_-] only. Example: "zero_crossing_count" 
    pub identifier: CString,
    /// The human-readable name of the output.
    /// 
    /// Example: "Zero Crossing Counts" 
    pub name: CString,
    /// A human-readable short text describing the output.
    /// 
    /// May be empty if the name has said it all already. Example: "The number of zero crossing points per processing block" 
    pub description: CString,
    /// The unit of the output, in human-readable form. 
    pub unit: CString,
    /// The number of values per result of the output, present if there is a fixed bin size
    /// 
    /// Undefined if hasFixedBinCount is false. If this is zero, the output is point data (i.e. only the time of each output is of interest, the value list will be empty). 
    pub bin_count: Option<usize>,
    /// The (human-readable) names of each of the bins, if appropriate. 
    pub bin_names: Option<Vec<CString>>,
    /// (Min,Max) True if the results in each output bin fall within a fixed numeric range (minimum and maximum values).
    pub extents: Option<(f32,f32)>,
    /// If present, resolution values are quantized to
    pub quantize_step: Option<f32>,
    pub sample_type: SampleType,
    pub has_duration: bool,
}

impl CxxOutputDescriptor {
    pub fn to_rust(&self) -> OutputDescriptor {
        OutputDescriptor::from(self)
    }
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
