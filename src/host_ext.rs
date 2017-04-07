cpp!{{
    #include <cstdint>
    #include <vector>
    #include <string>
    #include <vamp-hostsdk/vamp-hostsdk.h>
    using namespace Vamp::HostExt;
    using Vamp::Plugin;
    struct varray {
        void* a;
        int len;
    };
}}

    use std::ffi::{CStr,CString};
    use std::os::raw::c_char;

    #[test]
    fn test_get_instance() {
        let pl = unsafe {PluginLoader::get_instance()};
        println!("PluginLoader: {:#?}", pl);
        assert!(!pl.loader.is_null());
    }
    #[test]
    fn test_list_plugins() {
        let mut pl = unsafe {PluginLoader::get_instance()};
        assert!(!pl.loader.is_null());
        let out = pl.list_plugins();
        println!("PluginList: {:#?}", out);
    }
    #[test]
    fn test_compose_plugin_key() {
        let mut pl = unsafe {PluginLoader::get_instance()};
        assert!(!pl.loader.is_null());
        let out = pl.compose_plugin_key(CString::new("Foo").unwrap(),CString::new("Bar").unwrap());
        println!("PluginKey: {:#?}", out);
        assert!(out == CString::new("foo:Bar").unwrap());
    }
    #[test]
    fn test_get_plugin_category() {
        let mut pl = unsafe {PluginLoader::get_instance()};
        assert!(!pl.loader.is_null());
        let out = pl.get_plugin_category(CString::new("vamp-example-plugins:fixedtempo").unwrap());
        println!("PluginCategories: {:#?}", out);
    }
    #[test]
    fn test_get_library_path_for_plugin() {
        let mut pl = unsafe {PluginLoader::get_instance()};
        assert!(!pl.loader.is_null());
        let out = pl.get_library_path_for_plugin(CString::new("vamp-example-plugins:fixedtempo").unwrap());
        println!("{:#?}", out);
    }
    enum CxxPluginLoader {}
    #[derive(Debug)]
    pub struct PluginLoader {
        loader: *mut CxxPluginLoader
    }
    enum CxxPlugin {}
    #[derive(Debug)]
    pub struct Plugin {
        plugin: *mut CxxPlugin
    }
    type PluginCategoryHierarchy = Vec<CString>;

    type PluginKey = CString;
    impl PluginLoader {
        pub fn list_plugins(&mut self) -> Vec<PluginKey> {
            cppp!(conv_to_rust rust_data data_ptr (loader: "PluginLoader*" => self.loader) [let vec: Vec<CString> = Vec::new()] (c_str: *const c_char) {
                let cstr = unsafe {CStr::from_ptr(c_str)};
                vec.push(CString::from(cstr));
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
        /* pub fn list_plugins_in(&mut self, libraryNames: Vec<OsString>) -> Vec<PluginKey> {
            let mut plugin_list = Vec::new();
            c_rustfn!(conv_to_rust rust_data data_ptr [let vec: Vec<OsString> = plugin_list] (c_str: *const c_char) {
                let cstr = unsafe {CStr::from_ptr(c_str)};
                let bytes = OsStr::from_bytes(cstr.to_bytes());
                vec.push(OsString::from(bytes));
            });
            let mut c_str_vec: Vec<*const c_char> = Vec::new();
            for name in libraryNames {
                c_str_vec.push(name.as_os_str().as_bytes().as_ptr() as *const c_char);
            };
            let mut char_ptr: *const *const c_char = c_str_vec.as_ptr();
            let char_ptr_size = c_str_vec.len();
            let mut loader = self.loader;
            unsafe {
                cpp!([mut loader as "PluginLoader*", conv_to_rust as "void*", mut data_ptr as "void*", char_ptr as "char**", char_ptr_size as "uintptr_t"] {
                    auto strs = std::vector<std::string>(char_ptr_size);
                    for (int i = 0; i < char_ptr_size; i++) {
                        strs[i] = std::string(char_ptr[i]);
                    }
                    auto v = loader->listPluginsIn(strs); // MEMS
                    int jsj = 42;
                    for (int i = 0; i < v.size(); i++) {
                        ((void (*) (void*, const char *))conv_to_rust)(data_ptr,v[i].c_str()); // rust-cpp doesn't support function pointers in args it seems
                    }
                });
            }
            return rust_data.vec;
            unimplemented!();
        } */
        /* pub fn list_plugins_not_in(&mut self, libraryNames: Vec<String>) -> Vec<PluginKey> {
            unimplemented!();
        } */
        pub fn load_plugin(&mut self, key: PluginKey, input_sample_rate: f32, adapter_flags: i32) -> Option<Plugin> {
            let key_ptr = key.as_ptr();
            let mut loader = self.loader;
            match unsafe { cpp!([mut loader as "PluginLoader*", key_ptr as "char*", input_sample_rate as "float", adapter_flags as "int"] -> *mut CxxPlugin as "Plugin*" {
                    auto plugkey = std::string(key_ptr);
                    return loader->loadPlugin(plugkey, input_sample_rate, adapter_flags); // MEMS
                })} {
                a if !a.is_null() => Some(Plugin{ plugin: a }),
                _ => None
            }
        }
        pub fn compose_plugin_key(&mut self, library_name: CString, identifier: CString) -> PluginKey {
            cppp!(conv_to_rust rust_data data_ptr (loader: "PluginLoader*" => self.loader) [let out_key: Option<CString> = None] (c_str: *const c_char) {
                let cstr = unsafe {CStr::from_ptr(c_str)};
                *out_key = Some(CString::from(cstr));
            });
            let (lib_ptr, ident_ptr) = (library_name.as_ptr(), identifier.as_ptr());
            let mut loader = self.loader;
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
            let mut loader = self.loader;
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
            let mut loader = self.loader;
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
