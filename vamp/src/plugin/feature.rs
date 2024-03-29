use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use cxx_util::{CxxInnerVector, CxxVector};
pub enum CxxRealTimeInner {}
pub struct CxxRealTime(pub *mut CxxRealTimeInner);
#[derive(Debug, Clone)]
pub struct RealTime {
    /// The seconds component of the time value
    pub sec: i32,
    /// The nanoseconds component of the time value
    pub nsec: i32,
}

impl RealTime {
    pub fn new(sec: i32, nsec: i32) -> Self {
        Self {
            sec: sec,
            nsec: nsec,
        }
    }
    /// Create a rust RealTime object from a C++ reference
    pub fn from(ptr: *const CxxRealTime) -> Self {
        let sec = unsafe {
            cpp!([ptr as "Vamp::RealTime*"] -> i32 as "int" {
            return ptr->sec;
        })
        };
        let nsec = unsafe {
            cpp!([ptr as "Vamp::RealTime*"] -> i32 as "int" {
            return ptr->nsec;
        })
        };
        return RealTime {
            sec: sec,
            nsec: nsec,
        };
    }
}
impl CxxRealTime {
    pub fn from(feat: &RealTime) -> Self {
        let (sec, nsec) = (feat.sec, feat.nsec);
        unsafe {
            let ptr = cpp!([sec as "int", nsec as "int"] -> *mut CxxRealTimeInner as "Vamp::RealTime*" {
                return new Vamp::RealTime(sec, nsec);
            });
            CxxRealTime(ptr)
        }
    }
}
impl Drop for CxxRealTime {
    fn drop(&mut self) {
        let ptr = self.0 as *mut CxxRealTime;
        unsafe {
            cpp!([ptr as "Vamp::RealTime*"] {
            delete ptr;
        })
        };
    }
}

pub enum CxxFeature {}
#[derive(Debug)]
pub struct Feature {
    /// Timestamp of the output feature, if present.
    ///
    /// This is mandatory if the output has VariableSampleRate or if the output has FixedSampleRate and hasTimestamp is true, and unused otherwise.
    pub timestamp: Option<RealTime>,
    /// Duration of the output feature, if present.
    ///
    /// This is mandatory if the output has VariableSampleRate, can be present with FixedSampleRate, and unused otherwise.
    pub duration: Option<RealTime>,
    /// Results for a single sample of this feature.
    ///
    /// If the output hasFixedBinCount, there must be the same number of values as the output's binCount count.
    pub values: Vec<f32>,
    /// Label for the sample of this feature.
    pub label: CString,
}

impl CxxFeature {
    pub fn to_rust(&self) -> Feature {
        Feature::from(self)
    }
}

impl Feature {
    /// Create a rust Feature object from a C++ reference
    pub fn from(ptr: *const CxxFeature) -> Self {
        let has_timestamp = unsafe {
            cpp!([ptr as "Vamp::Plugin::Feature*"] -> bool as "bool" {
            return ptr->hasTimestamp;
        })
        };
        let timestamp: Option<RealTime> = match has_timestamp {
            true => {
                let tstamp_ptr = unsafe {
                    cpp!([ptr as "Vamp::Plugin::Feature*"] -> *const CxxRealTime as "const Vamp::RealTime*" {
                    return &(ptr->timestamp);
                })
                };
                Some(RealTime::from(tstamp_ptr))
            }
            false => None,
        };
        let has_duration = unsafe {
            cpp!([ptr as "Vamp::Plugin::Feature*"] -> bool as "bool" {
            return ptr->hasDuration;
        })
        };
        let duration: Option<RealTime> = match has_duration {
            true => {
                let duration_ptr = unsafe {
                    cpp!([ptr as "Vamp::Plugin::Feature*"] -> *const CxxRealTime as "const Vamp::RealTime*" {
                    return &(ptr->duration);
                })
                };
                Some(RealTime::from(duration_ptr))
            }
            false => None,
        };
        let values_v: CxxVector<f32> = unsafe {
            CxxVector::from(
                cpp!([ptr as "Vamp::Plugin::Feature*"] -> *mut CxxInnerVector as "const std::vector<float>*" {
            return &(ptr->values);
        }),
            )
        };
        let values = values_v.to_vec();
        values_v.into_raw();

        let label = unsafe {
            CStr::from_ptr(
                cpp!([ptr as "Vamp::Plugin::Feature*"] -> *const c_char as "const char*" {
            return ptr->label.c_str();
        }),
            )
        }.to_owned();
        return Feature {
            timestamp: timestamp,
            duration: duration,
            values: values,
            label: label,
        };
    }
}
