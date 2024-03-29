#![feature(trace_macros)]
#[link(name = "vamp-hostsdk")]
extern {}
#[macro_use]
extern crate cpp;
#[macro_use]
extern crate cpp_macros;

macro_rules! c_rustfn {
    ($fn_name:ident $data_name:ident $data_ptr_name:ident [$(let $p_name:ident : $p_type:ty = $var:expr ),+] ($($c_name:ident : $c_type:ty),*) $body:block) => {
        struct FnData {
           $($p_name : $p_type),+
        }
        extern "C" fn rust_fn( data: *mut FnData, $($c_name:$c_type),* ) {
            not_null!(data);
            let mut dataref = unsafe { &mut *data };
            $(let ref mut $p_name = dataref.$p_name;)*
            $body
        }
        let $fn_name = rust_fn as *mut fn(*mut FnData, $($c_type),*);
        let mut $data_name = FnData {
            $($p_name : $var),+
        };
        let mut $data_ptr_name: *mut FnData = &mut $data_name;
    }
}
macro_rules! not_null {
    ($($var:ident),+) => {
        $(
            assert!(!$var.is_null());
        )+
    };
}

#[cfg(test)]
mod tests; 

mod host_ext;
mod plugin;
mod cxx_util;

pub use host_ext::{PluginLoader,PluginKey,PluginCategoryHierarchy};
pub use plugin::{Plugin,Feature,FeatureList,FeatureSet,OutputDescriptor,OutputList,ParameterDescriptor,ParameterList,RealTime,InputDomain,SampleType};
