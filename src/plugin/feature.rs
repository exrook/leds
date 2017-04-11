use std::ffi::{CStr,CString};
use std::os::raw::c_char;
use ::cxx_util::{CxxInnerVector,CxxVector};
pub enum CxxRealTime {}
pub struct RealTime {
    sec: i32,
    nsec: i32
}

impl RealTime {
    pub fn from(ptr: *const CxxRealTime) -> Self {
        let sec = unsafe { cpp!([ptr as "Vamp::RealTime*"] -> i32 as "int" {
            return ptr->sec;
        })};
        let nsec = unsafe { cpp!([ptr as "Vamp::RealTime*"] -> i32 as "int" {
            return ptr->nsec;
        })};
        return RealTime {
            sec: sec,
            nsec: nsec
        }
    }
}

pub enum CxxFeature {}
pub struct Feature {
    timestamp: Option<RealTime>,
    duration: Option<RealTime>,
    values: Vec<f32>,
    label: CString
}

impl CxxFeature {
    pub fn to_rust(&self) -> Feature {
        Feature::from(self)
    }
}

impl Feature { 
    pub fn from(ptr: *const CxxFeature) -> Self {
        let has_timestamp = unsafe { cpp!([ptr as "Vamp::Plugin::Feature*"] -> bool as "bool" {
            return ptr->hasTimestamp;
        })};
        let timestamp: Option<RealTime> = match has_timestamp {
            true => {
                let tstamp_ptr = unsafe { cpp!([ptr as "Vamp::Plugin::Feature*"] -> *const CxxRealTime as "const Vamp::RealTime*" {
                    return &(ptr->timestamp);
                })};
                unimplemented!();
            }
            false => {
                None
            }
        };
        let has_duration = unsafe { cpp!([ptr as "Vamp::Plugin::Feature*"] -> bool as "bool" {
            return ptr->hasDuration;
        })};
        let duration: Option<RealTime> = match has_duration {
            true => {
                let duration_ptr = unsafe { cpp!([ptr as "Vamp::Plugin::Feature*"] -> *const CxxRealTime as "const Vamp::RealTime*" {
                    return &(ptr->duration);
                })};
                unimplemented!();
            }
            false => {
                None
            }
        };
        let values_v: CxxVector<f32> = unsafe { CxxVector::from(cpp!([ptr as "Vamp::Plugin::Feature*"] -> *mut CxxInnerVector as "const std::vector<float>*" {
            return &(ptr->values);
        }))};
        let values = values_v.to_vec();
        values_v.into_raw();

        let label = unsafe { CStr::from_ptr(cpp!([ptr as "Vamp::Plugin::Feature*"] -> *const c_char as "const char*" {
            return ptr->label.c_str();
        }))}.to_owned();
        return Feature {
            timestamp: timestamp,
            duration: duration,
            values: values,
            label: label
        }
    }
}
