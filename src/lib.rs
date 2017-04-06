#![feature(trace_macros)]
#[macro_use]
extern crate cpp;
#[macro_use]
extern crate cpp_macros;


cpp!{{
    #include <cstdint>
    #include <vamp-hostsdk/vamp-hostsdk.h>
    using namespace Vamp::HostExt;
    struct varray {
        void* a;
        int len;
    };
}}

#[cfg(test)]
mod tests {
    use ::vamp_host::PluginLoader;
    #[test]
    fn it_works() {
    }
    #[test]
    fn test_get_instance() {
        let pl = unsafe {PluginLoader::get_instance()};
        println!("Oh shit whaddup");
        println!("{:#?}", pl);
    }
    //#[test]
    //fn test_list_plugins() {
    //    let mut pl = unsafe {PluginLoader::get_instance()};
    //    assert!(!pl.loader.is_null());
    //    println!("{:#?}", pl);
    //    println!("Oh hey it's");
    //    let out = pl.list_plugins();
    //    println!("{:#?}", out);
    //}
}

macro_rules! c_rustfn {
    ($fn_name:ident $data_name:ident $data_ptr_name:ident [$(let $p_name:ident : $p_type:ty = $var:ident ),+] ($($c_name:ident : $c_type:ty),*) $body:block) => {
        struct dfs {
           $($p_name : $p_type),+ 
        }
        extern "C" fn cfjds( mut envv: *mut dfs, $($c_name:$c_type),* ) {
            not_null!(envv);
            let mut envvv = unsafe { &mut *envv };
            $(let ref mut $p_name = envvv.$p_name;)*
            $body
        }
        let $fn_name = cfjds as *mut fn(*mut dfs, $($c_type),*);
        let mut $data_name = dfs {
            $($p_name : $var),+
        };
        let mut $data_ptr_name: *mut dfs = &mut $data_name;
    }
}
macro_rules! not_null {
    ($($var:ident),+) => {
        $(
            assert!(!$var.is_null());
        )+
    };
    ($($var:ident : $lul:ty),+) => {
        not_null!($var)
    }
}


mod vamp_host {
    use std::ffi::{OsStr,OsString};
    use std::ffi::{CStr,CString};
    use std::os::unix::ffi::OsStrExt;
    use std::os::raw::c_char;

    #[test]
    fn test_list_plugins() {
        let mut pl = unsafe {PluginLoader::get_instance()};
        assert!(!pl.loader.is_null());
        println!("{:#?}", pl);
        println!("Oh hey it's");
        let out = pl.list_plugins();
        println!("{:#?}", out);
    }
    enum CxxPluginLoader {}
    #[derive(Debug)]
    pub struct PluginLoader {
       loader: *mut CxxPluginLoader
    }
    pub enum Plugin {}
    pub enum PluginCategoryHierarchy {}

    type PluginKey = OsString;
    impl PluginLoader {
        pub fn list_plugins(&mut self) -> Vec<PluginKey> {
            let mut plugin_list = Vec::new();
            c_rustfn!(conv_to_rust rust_data data_ptr [let vec: Vec<OsString> = plugin_list] (whaddup: *const c_char) {
                let cstr = unsafe {CStr::from_ptr(whaddup)};
                let bytes = OsStr::from_bytes(cstr.to_bytes());
                vec.push(OsString::from(bytes));
            });
            let mut loader = self.loader;
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
        pub fn list_plugins_in(&mut self, libraryNames: Vec<String>) -> Vec<PluginKey> {
            unimplemented!();
        }
        pub fn list_plugins_not_in(&mut self, libraryNames: Vec<String>) -> Vec<PluginKey> {
            unimplemented!();
        }
        pub fn load_plugin(&mut self, key: PluginKey, inputSampleRate: f32, adapterFlags: i32) -> Plugin {
            unimplemented!();
        }
        pub fn compose_plugin_key(&mut self, libraryName: String, identifier: String) -> PluginKey {
            unimplemented!();
        }
        pub fn get_plugin_category(&mut self, plugin: PluginKey) -> PluginCategoryHierarchy {
            unimplemented!();
        }
        pub fn get_library_path_for_plugin(&mut self, plugin: PluginKey) -> String {
            unimplemented!();
        }
        /// Only call this once, if you have to call it more than once u need to re-evaluate life
        pub unsafe fn get_instance() -> PluginLoader {
            let load = cpp!( [] -> *mut CxxPluginLoader as "PluginLoader*" {
                return PluginLoader::getInstance();
            });
            PluginLoader {
                loader: load
            }
        }
    }
   // cpp!([] {

   // });
}
