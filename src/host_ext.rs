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
use plugin::Plugin as Plugin;

#[test]
fn test_get_instance() {
    let pl = unsafe {PluginLoader::get_instance()};
    println!("Got plugin loader: {:#?}", pl as *const _);
}
#[test]
fn test_list_plugins() {
    let mut pl = unsafe {PluginLoader::get_instance()};
    let out = pl.list_plugins();
    println!("PluginList: {:#?}", out);
}
#[test]
fn test_load_plugin() {
    let mut pl = unsafe {PluginLoader::get_instance()};
    let out = pl.load_plugin(CString::new("vamp-example-plugins:fixedtempo").unwrap(), 44100.0, 0x03).unwrap();
    let raw_ptr = Box::into_raw(out);
    println!("Loaded Plugin: {:#?}", raw_ptr);
    let out = unsafe {Box::from_raw(raw_ptr)};
}
#[test]
fn test_compose_plugin_key() {
    let mut pl = unsafe {PluginLoader::get_instance()};
    let out = pl.compose_plugin_key(CString::new("Foo").unwrap(),CString::new("Bar").unwrap());
    println!("PluginKey: {:#?}", out);
    assert!(out == CString::new("foo:Bar").unwrap());
}
#[test]
fn test_get_plugin_category() {
    let mut pl = unsafe {PluginLoader::get_instance()};
    let out = pl.get_plugin_category(CString::new("vamp-example-plugins:fixedtempo").unwrap());
    println!("PluginCategories: {:#?}", out);
}
#[test]
fn test_get_library_path_for_plugin() {
    let mut pl = unsafe {PluginLoader::get_instance()};
    let out = pl.get_library_path_for_plugin(CString::new("vamp-example-plugins:fixedtempo").unwrap());
    println!("{:#?}", out);
}
pub enum PluginLoader {}
type PluginCategoryHierarchy = Vec<CString>;

type PluginKey = CString;
impl PluginLoader {
    pub fn list_plugins(&mut self) -> Vec<PluginKey> {
        cppp!(conv_to_rust rust_data data_ptr (loader: "PluginLoader*" => self.loader) [let vec: Vec<CString> = Vec::new()] (c_str: *const c_char) {
            let cstr = unsafe {CStr::from_ptr(c_str)};
            vec.push(CString::from(cstr));
        });
        let mut loader = self as *mut _;
        unsafe {
            cpp!([mut loader as "PluginLoader*", conv_to_rust as "void*", mut data_ptr as "void*"] {
                auto v = loader->listPlugins();
                for (int i = 0; i < v.size(); i++) {
                    ((void (*) (void*, const char *))conv_to_rust)(data_ptr,v[i].c_str()); // rust-cpp doesn't support function pointers in args it seems
                }
            });
        }
        return rust_data.vec;
    }
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
    pub fn compose_plugin_key(&mut self, library_name: CString, identifier: CString) -> PluginKey {
        cppp!(conv_to_rust rust_data data_ptr (loader: "PluginLoader*" => self.loader) [let out_key: Option<CString> = None] (c_str: *const c_char) {
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
    pub fn get_plugin_category(&mut self, plugin: PluginKey) -> PluginCategoryHierarchy {
        cppp!(conv_to_rust rust_data data_ptr (loader: "PluginLoader*" => self.loader) [let vec: Vec<CString> = Vec::new()] (c_str: *const c_char) {
            let cstr = unsafe {CStr::from_ptr(c_str)};
            vec.push(CString::from(cstr));
        });
        let plug_ptr = plugin.as_ptr();
        let mut loader = self as *mut _;
        unsafe {
            cpp!([mut loader as "PluginLoader*", conv_to_rust as "void*", mut data_ptr as "void*", plug_ptr as "char*"] {
                auto plug = std::string(plug_ptr);
                auto v = loader->getPluginCategory(plug);
                for (int i = 0; i < v.size(); i++) {
                    ((void (*) (void*, const char *))conv_to_rust)(data_ptr,v[i].c_str()); // rust-cpp doesn't support function pointers in args it seems
                }
            });
        }
        rust_data.vec
    }
    pub fn get_library_path_for_plugin(&mut self, plugin: PluginKey) -> CString {
        cppp!(conv_to_rust rust_data data_ptr (loader: "PluginLoader*" => self.loader) [let out_path: Option<CString> = None] (c_str: *const c_char) {
            let cstr = unsafe {CStr::from_ptr(c_str)};
            *out_path = Some(CString::from(cstr));
        });
        let plug_ptr = plugin.as_ptr();
        let mut loader = self as *mut _;
        unsafe {
            cpp!([mut loader as "PluginLoader*", conv_to_rust as "void*", mut data_ptr as "void*", plug_ptr as "char*"] {
                auto plug = std::string(plug_ptr);
                auto v = loader->getLibraryPathForPlugin(plug);
                ((void (*) (void*, const char *))conv_to_rust)(data_ptr,v.c_str()); // rust-cpp doesn't support function pointers in args it seems
            });
        }
        rust_data.out_path.unwrap()
    }
    /// Only call this once, if you have to call it more than once u need to re-evaluate life
    pub unsafe fn get_instance() -> &'static mut PluginLoader {
        let load = cpp!( [] -> *mut PluginLoader as "PluginLoader*" {
            return PluginLoader::getInstance();
        });
        not_null!(load);
        &mut *load
    }
}
// cpp!([] {

// });
