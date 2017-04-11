use std::ffi::CString;
use std::collections::BTreeMap;

use ::cxx_util::{CxxVector,CxxInnerVector,CxxString};

mod output_descriptor;
mod feature;
mod parameter_descriptor;
pub use self::feature::Feature;
pub use self::feature::CxxFeature;
pub use self::output_descriptor::{CxxOutputDescriptor,OutputDescriptor};
pub use self::parameter_descriptor::{CxxParameterDescriptor,ParameterDescriptor};
type FeatureList = Vec<Feature>;
type FeatureSet = BTreeMap<i32, FeatureList>;
type ProgramList = Vec<CString>;
type ParameterList = Vec<ParameterDescriptor>;
pub enum Plugin {}
pub enum InputDomain {
    TimeDomain,
    FrequencyDomain
}
impl Plugin {
    pub fn initialise(&mut self, input_channels: usize, step_size: usize, block_size: usize) -> Result<(),()> {
        let mut plugin = self as *mut _;
        match unsafe { cpp!([mut plugin as "Plugin*", input_channels as "size_t", step_size as "size_t", block_size as "size_t"] -> bool as "bool" {
                return plugin->initialise(input_channels, step_size, block_size); // MEMS
            })} {
            true => Ok(()),
            false => Err(())
        }
    }
    pub fn reset(&mut self) {
        let mut plugin = self as *mut _;
        unsafe { cpp!([mut plugin as "Plugin*"] { plugin->reset(); })}
    }
    pub fn get_input_domain(&self) -> InputDomain {
        let mut plugin = self as *const _;
        match unsafe { cpp!([plugin as "Plugin*"] -> i32 as "int" {
                auto en = plugin->getInputDomain(); // MEMS
                switch (en) {
                    case Plugin::TimeDomain:
                        return 0;
                    case Plugin::FrequencyDomain:
                        return 1;
                }
            })} {
            0 => InputDomain::TimeDomain,
            1 => InputDomain::FrequencyDomain,
            _ => unreachable!()
        }
    }
    pub fn get_preferred_block_size(&self) -> usize {
        let plugin = self as *const _;
        unsafe { cpp!([plugin as "Plugin*"] -> usize as "size_t" {
            return plugin->getPreferredBlockSize(); // MEMS
        })}
    }
    pub fn get_preferred_step_size(&self) -> usize {
        let plugin = self as *const _;
        unsafe { cpp!([plugin as "Plugin*"] -> usize as "size_t" {
            return plugin->getPreferredStepSize(); // MEMS
        })}
    }
    pub fn get_min_channel_count(&self) -> usize {
        let plugin = self as *const _;
        unsafe { cpp!([plugin as "Plugin*"] -> usize as "size_t" {
            return plugin->getMinChannelCount(); // MEMS
        })}
    }
    pub fn get_max_channel_count(&self) -> usize {
        let plugin = self as *const _;
        unsafe { cpp!([plugin as "Plugin*"] -> usize as "size_t" {
            return plugin->getMaxChannelCount(); // MEMS
        })}
    }
    pub fn get_output_descriptors(&self) -> Vec<OutputDescriptor> {
        let plugin = self as *const _;
        let tmp: CxxVector<CxxOutputDescriptor> = unsafe { CxxVector::from(cpp!([plugin as "Plugin*"] -> *mut CxxInnerVector as "std::vector<Plugin::OutputDescriptor>*" {
            auto out = new std::vector<Plugin::OutputDescriptor>();
            *out = plugin->getOutputDescriptors();
            return out;
        }))};
        let v = tmp.to_vec();
        unsafe {tmp.delete()};
        v
    }
    pub fn process(&mut self) -> FeatureSet {
        unimplemented!();
    }
    pub fn get_remaining_features(&mut self) -> FeatureSet {
        unimplemented!();
    }
    pub fn get_type(&self) -> CString {
        let plugin = self as *const _;
        let s = unsafe { cpp!([plugin as "Plugin*"] -> *mut CxxString as "std::string*" {
                auto out = new std::string();
                *out = plugin->getType();
                return out;
            }) };
        not_null!(s);
        let s = unsafe { &mut *s };
        let out = s.to_c_string();
        unsafe {s.delete()};
        return out;
    }
    pub fn get_vamp_api_version(&self) -> u32 {
        let plugin = self as *const _;
        unsafe { cpp!([plugin as "Plugin*"] -> u32 as "uint" {
            return plugin->getVampApiVersion(); // MEMS
        })}
    }
    pub fn get_identifier(&self) -> CString {
        let plugin = self as *const _;
        let s = unsafe { cpp!([plugin as "Plugin*"] -> *mut CxxString as "std::string*" {
                auto out = new std::string();
                *out = plugin->getIdentifier();
                return out;
            }) };
        not_null!(s);
        let s = unsafe { &mut *s };
        let out = s.to_c_string();
        unsafe {s.delete()};
        return out;
    }
    pub fn get_name(&self) -> CString {
        let plugin = self as *const _;
        let s = unsafe { cpp!([plugin as "Plugin*"] -> *mut CxxString as "std::string*" {
                auto out = new std::string();
                *out = plugin->getName();
                return out;
            }) };
        not_null!(s);
        let s = unsafe { &mut *s };
        let out = s.to_c_string();
        unsafe {s.delete()};
        return out;
    }
    pub fn get_description(&self) -> CString {
        let plugin = self as *const _;
        let s = unsafe { cpp!([plugin as "Plugin*"] -> *mut CxxString as "std::string*" {
                auto out = new std::string();
                *out = plugin->getDescription();
                return out;
            }) };
        not_null!(s);
        let s = unsafe { &mut *s };
        let out = s.to_c_string();
        unsafe {s.delete()};
        return out;
    }
    pub fn get_maker(&self) -> CString {
        let plugin = self as *const _;
        let s = unsafe { cpp!([plugin as "Plugin*"] -> *mut CxxString as "std::string*" {
                auto out = new std::string();
                *out = plugin->getMaker();
                return out;
            }) };
        not_null!(s);
        let s = unsafe { &mut *s };
        let out = s.to_c_string();
        unsafe {s.delete()};
        return out;
    }
    pub fn get_copyright(&self) -> CString {
        let plugin = self as *const _;
        let s = unsafe { cpp!([plugin as "Plugin*"] -> *mut CxxString as "std::string*" {
                auto out = new std::string();
                *out = plugin->getCopyright();
                return out;
            }) };
        not_null!(s);
        let s = unsafe { &mut *s };
        let out = s.to_c_string();
        unsafe {s.delete()};
        return out;
    }
    pub fn get_plugin_version(&self) -> u32 {
        let mut plugin = self as *const _;
        unsafe { cpp!([plugin as "Plugin*"] -> u32 as "uint" {
            return plugin->getPluginVersion(); // MEMS
        })}
    }
    pub fn get_parameter_descriptors(&self) -> ParameterList {
        unimplemented!();
    }
    pub fn get_parameter(&self, param: CString) -> f32 {
        unimplemented!();
    }
    pub fn set_parameter(&mut self, param: CString, value: f32) {
        unimplemented!();
    }
    pub fn get_programs(&self) -> ProgramList {
        unimplemented!();
    }
    pub fn get_current_program(&self) -> CString {
        let plugin = self as *const _;
        let s = unsafe { cpp!([plugin as "Plugin*"] -> *mut CxxString as "std::string*" {
                auto out = new std::string();
                *out = plugin->getCurrentProgram();
                return out;
            }) };
        not_null!(s);
        let s = unsafe { &mut *s };
        let out = s.to_c_string();
        unsafe {s.delete()};
        return out;
    }
    pub fn select_program(&self, program: CString) {
        unimplemented!();
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        let mut plugin = self as *mut _;
        unsafe { cpp!([mut plugin as "Plugin*"] {
            delete plugin;
        })}
    }
}
