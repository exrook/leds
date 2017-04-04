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
            unsafe {
                enum CxxVec {}
                let mut loader = self.loader;
                let mut size: i32 = 0;
                let mut v = cpp!([mut loader as "PluginLoader*", mut size as "int32_t"] -> *mut CxxVec as "std::vector<std::string>* " {
                    auto v = new std::vector<std::string>(loader->listPlugins());
                    size = v->size();
                    return v;
                });
                println!("Vector: {:#?}", v);
                let mut str_vec: Vec<*const c_char> = Vec::with_capacity(size as usize);
                let str_vec_ptr = str_vec.as_mut_ptr();
                let len = cpp!([mut v as "std::vector<std::string>* ", size as "int32_t", str_vec_ptr as "char**"] {
                    for (int i = 0; i < size; i++) {
                        str_vec_ptr[i] = (*v)[i].c_str();
                    }
                });
                str_vec.set_len(size as usize);
                for s in str_vec {
                    let cstr = CStr::from_ptr(s);
                    let bytes = OsStr::from_bytes(cstr.to_bytes());
                    plugin_list.push(OsString::from(bytes));
                }
                cpp!([v as "std::vector<std::string>* "] {
                    delete v;
                });
            }
            return plugin_list;
        }
        fn list_plugins_in(&mut self, libraryNames: Vec<String>) -> Vec<PluginKey> {
            unimplemented!();
        }
        fn list_plugins_not_in(&mut self, libraryNames: Vec<String>) -> Vec<PluginKey> {
            unimplemented!();
        }
        fn load_plugin(&mut self, key: PluginKey, inputSampleRate: f32, adapterFlags: i32) -> Plugin {
            unimplemented!();
        }
        fn compose_plugin_key(&mut self, libraryName: String, identifier: String) -> PluginKey {
            unimplemented!();
        }
        fn get_plugin_category(&mut self, plugin: PluginKey) -> PluginCategoryHierarchy {
            unimplemented!();
        }
        fn get_library_path_for_plugin(&mut self, plugin: PluginKey) -> String {
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
