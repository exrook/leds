cpp!{{
    #include <cstdint>
    #include <vector>
    #include <string>
    #include <vamp-hostsdk/vamp-hostsdk.h>
    using namespace Vamp::HostExt;
    using Vamp::Plugin;
}}

use std::ffi::{CStr,CString};
use std::os::raw::c_char;
use std::sync::{Arc,Mutex};
use plugin::Plugin as Plugin;
use ::cxx_util::{CxxString,CxxVector,CxxInnerVector};

static mut HOSTEXT: &'static Option<Arc<Mutex<Box<PluginLoader>>>> = &None;

pub enum PluginLoader {}
type PluginCategoryHierarchy = Vec<CString>;

type PluginKey = CString;
impl PluginLoader {
    /// True if the returned results for this output are known to have a duration field.
    pub fn list_plugins(&mut self) -> Vec<PluginKey> {
        let mut loader = self as *mut _;
        let cxxvec: CxxVector<CxxString> = unsafe {CxxVector::from(cpp!([mut loader as "PluginLoader*"] -> *mut CxxInnerVector as "std::vector<std::string>*" {
            auto vv = new std::vector<std::string>();
            *vv = loader->listPlugins();
            return vv;
        }))};
        let out = cxxvec.to_vec();
        unsafe{cxxvec.delete()};
        return out;
    }
    /// Load a Vamp plugin, given its identifying key.
    ///
    /// If the plugin could not be loaded, returns 0.
    ///
    /// The returned plugin should be deleted (using the standard C++ delete keyword) after use.
    pub fn load_plugin(&mut self, key: PluginKey, input_sample_rate: f32, adapter_flags: i32) -> Option<Box<Plugin>> {
        let key_ptr = key.as_ptr();
        let mut loader = self as *mut _;
        match unsafe { cpp!([mut loader as "PluginLoader*", key_ptr as "char*", input_sample_rate as "float", adapter_flags as "int"] -> *mut Plugin as "Plugin*" {
                auto plugkey = std::string(key_ptr);
                return loader->loadPlugin(plugkey, input_sample_rate, adapter_flags); // MEMS
            })} {
            a if !a.is_null() => Some(unsafe{Box::from_raw( a )}),
            _ => None
        }
    }
    /// Given a Vamp plugin library name and plugin identifier, return the corresponding plugin key in a form suitable for passing in to loadPlugin().
    pub fn compose_plugin_key(&mut self, library_name: CString, identifier: CString) -> PluginKey {
        c_rustfn!(conv_to_rust rust_data data_ptr [let out_key: Option<CString> = None] (c_str: *const c_char) {
            let cstr = unsafe {CStr::from_ptr(c_str)};
            *out_key = Some(CString::from(cstr));
        });
        let (lib_ptr, ident_ptr) = (library_name.as_ptr(), identifier.as_ptr());
        let mut loader = self as *mut _;
        unsafe {
            cpp!([mut loader as "PluginLoader*", conv_to_rust as "void*", mut data_ptr as "void*", lib_ptr as "char*", ident_ptr as "char*"] {
                auto lib = std::string(lib_ptr);
                auto ident = std::string(ident_ptr);
                auto v = loader->composePluginKey(lib,ident);
                ((void (*) (void*, const char *))conv_to_rust)(data_ptr,v.c_str()); // rust-cpp doesn't support function pointers in args it seems
            });
        }
        rust_data.out_key.unwrap()
    }
    /// Return the category hierarchy for a Vamp plugin, given its identifying key.
    ///
    /// If the plugin has no category information, return an empty hierarchy.
    pub fn get_plugin_category(&mut self, plugin: PluginKey) -> PluginCategoryHierarchy {
        let plug_ptr = plugin.as_ptr();
        let mut loader = self as *mut _;
        let cxxvec: CxxVector<CxxString> = unsafe {CxxVector::from(cpp!([mut loader as "PluginLoader*", plug_ptr as "char*"] -> *mut CxxInnerVector as "std::vector<std::string>*" {
            auto plug = std::string(plug_ptr);
            auto v = new std::vector<std::string>();
            *v = loader->getPluginCategory(plug);
            return v;
        }))};
        let out = cxxvec.to_vec();
        unsafe { cxxvec.delete() };
        return out;
    }
    /// Return the file path of the dynamic library from which the given plugin will be loaded (if available).
    pub fn get_library_path_for_plugin(&mut self, plugin: PluginKey) -> CString {
        let plug_ptr = plugin.as_ptr();
        let mut loader = self as *mut _;
        let s = unsafe { cpp!([mut loader as "PluginLoader*", plug_ptr as "char*"] -> *mut CxxString as "std::string*" {
                auto plug = std::string(plug_ptr);
                auto out = new std::string();
                *out = loader->getLibraryPathForPlugin(plug);
                return out;
            }) };
        not_null!(s);
        let s = unsafe { &mut *s };
        let out = s.to_c_string();
        unsafe {s.delete()};
        return out;
    }
    /// Returns a reference to the PluginLoader singleton, wrapped in a mutex. Try not to use this
    /// from multiple threads, even though there is a mutex, as the C++ doesn't seem to support it
    pub unsafe fn get_instance() -> Arc<Mutex<Box<PluginLoader>>> {
        match HOSTEXT {
            &None => {
                let load = cpp!( [] -> *mut PluginLoader as "PluginLoader*" {
                    return PluginLoader::getInstance();
                });
                not_null!(load);
                let tmp = Box::new(Some(Arc::new(Mutex::new(Box::from_raw(load)))));
                HOSTEXT = &mut *Box::into_raw(tmp);
                return PluginLoader::get_instance();
            }
            &Some(ref a) => {
                return a.clone();
            }
        }
    }
}
// cpp!([] {

// });
