use std::ffi::CString;
use std::collections::BTreeMap;

use ::cxx_util::{CxxVector,CxxInnerVector,CxxMap,CxxInnerMap,CxxString};

mod output_descriptor;
mod feature;
mod parameter_descriptor;
pub use self::feature::{Feature,RealTime};
pub use self::feature::{CxxFeature,CxxRealTime};
pub use self::output_descriptor::{CxxOutputDescriptor,OutputDescriptor};
pub use self::parameter_descriptor::{CxxParameterDescriptor,ParameterDescriptor};
type FeatureList = Vec<Feature>;
type FeatureSet = BTreeMap<i32, FeatureList>;
type ProgramList = Vec<CString>;
type ParameterList = Vec<ParameterDescriptor>;
pub enum Plugin {}
#[derive(Debug)]
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
    pub fn process(&mut self, input_buffers: Vec<Vec<f32>>, timestamp: RealTime) -> FeatureSet {
        let mut plugin = self as *mut _;
        let mut c_input_bufs = Vec::new();
        for b in input_buffers {
            c_input_bufs.push(b.as_ptr());
        }
        let c_input_bufs_ptr = c_input_bufs.as_ptr();
        let tstamp_box = CxxRealTime::from(&timestamp);
        let tstamp_ptr = Box::into_raw(tstamp_box);
        let tmp: CxxMap<i32,CxxVector<CxxFeature>> = unsafe { CxxMap::from(cpp!([mut plugin as "Plugin*", c_input_bufs_ptr as "float *const *", tstamp_ptr as "Vamp::RealTime*"] -> *mut CxxInnerMap as "std::map<int,std::vector<Plugin::Feature>>*" {
            auto out = new std::map<int,std::vector<Plugin::Feature>>();
            *out = plugin->process(c_input_bufs_ptr, *tstamp_ptr);
            return out;
        }))};
        let tstamp_box = unsafe{Box::from_raw(tstamp_ptr)};
        let m = tmp.to_map();
        unsafe {tmp.delete()};
        m
    }
    pub fn get_remaining_features(&mut self) -> FeatureSet {
        let mut plugin = self as *mut _;
        let tmp: CxxMap<i32,CxxVector<CxxFeature>> = unsafe { CxxMap::from(cpp!([mut plugin as "Plugin*"] -> *mut CxxInnerMap as "std::map<int,std::vector<Plugin::Feature>>*" {
            auto out = new std::map<int,std::vector<Plugin::Feature>>();
            *out = plugin->getRemainingFeatures();
            return out;
        }))};
        let m = tmp.to_map();
        unsafe {tmp.delete()};
        m
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
        let mut plugin = self as *const _;
        let cxxvec: CxxVector<CxxParameterDescriptor> = unsafe {CxxVector::from(cpp!([mut plugin as "Plugin*"] -> *mut CxxInnerVector as "std::vector<Plugin::ParameterDescriptor>*" {
            auto vv = new std::vector<Plugin::ParameterDescriptor>();
            *vv = plugin->getParameterDescriptors();
            return vv;
        }))};
        let out = cxxvec.to_vec();
        unsafe{cxxvec.delete()};
        return out;
    }
    pub fn get_parameter(&self, param: CString) -> f32 {
        let param_ptr = param.as_ptr();
        let plugin = self as *const _;
        unsafe { cpp!([plugin as "Plugin*", param_ptr as "char*"] -> f32 as "float" {
            auto param = std::string(param_ptr);
            return plugin->getParameter(param); // MEMS
        })}
    }
    pub fn set_parameter(&mut self, param: CString, value: f32) {
        let param_ptr = param.as_ptr();
        let mut plugin = self as *mut _;
        unsafe { cpp!([mut plugin as "Plugin*", param_ptr as "char*", value as "float"] {
            auto param = std::string(param_ptr);
            plugin->setParameter(param, value); // MEMS
        })}
    }
    pub fn get_programs(&self) -> ProgramList {
        let plugin = self as *const _;
        let cxxvec: CxxVector<CxxString> = unsafe {CxxVector::from(cpp!([plugin as "Plugin*"] -> *mut CxxInnerVector as "std::vector<std::string>*" {
            auto v = new std::vector<std::string>();
            *v = plugin->getPrograms();
            return v;
        }))};
        let out = cxxvec.to_vec();
        unsafe { cxxvec.delete() };
        return out;
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
    pub fn select_program(&mut self, program: CString) {
        let prog_ptr = program.as_ptr();
        let mut plugin = self as *mut _;
        let s = unsafe { cpp!([mut plugin as "Plugin*", prog_ptr as "char*"] {
            auto prog = std::string(prog_ptr);
            plugin->selectProgram(prog);
        }) };
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
