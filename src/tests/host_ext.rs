use std::ffi::CString;
use ::PluginLoader;
#[test]
fn test_get_instance() {
    let pl = unsafe {PluginLoader::get_instance()};
    println!("Waiting for lock");
    let pl = pl.lock().unwrap();
    println!("Got lock");
    println!("Got plugin loader: {:p}", *pl);
}
#[test]
fn test_list_plugins() {
    let pl = unsafe {PluginLoader::get_instance()};
    println!("Waiting for lock");
    let mut pl = pl.lock().unwrap();
    println!("Got lock test_list_plugins");
    let out = pl.list_plugins();
    println!("PluginList: {:#?}", out);
}
#[test]
fn test_load_plugin() {
    let pl = unsafe {PluginLoader::get_instance()};
    println!("Waiting for lock");
    let mut pl = pl.lock().unwrap();
    println!("Got lock test_load_plugin");
    let out = pl.load_plugin(CString::new("vamp-example-plugins:fixedtempo").unwrap(), 44100.0, 0x03).unwrap();
    let raw_ptr = Box::into_raw(out);
    println!("Loaded Plugin: {:#?}", raw_ptr);
    let out = unsafe {Box::from_raw(raw_ptr)};
}
#[test]
fn test_compose_plugin_key() {
    let pl = unsafe {PluginLoader::get_instance()};
    println!("Waiting for lock");
    let mut pl = pl.lock().unwrap();
    println!("Got lock compose_plugin_key");
    let out = pl.compose_plugin_key(CString::new("Foo").unwrap(),CString::new("Bar").unwrap());
    println!("PluginKey: {:#?}", out);
    assert!(out == CString::new("foo:Bar").unwrap());
}
#[test]
fn test_get_plugin_category() {
    let pl = unsafe {PluginLoader::get_instance()};
    println!("Waiting for lock");
    let mut pl = pl.lock().unwrap();
    println!("Got lock get_plugin_category");
    let out = pl.get_plugin_category(CString::new("vamp-example-plugins:fixedtempo").unwrap());
    println!("PluginCategories: {:#?}", out);
}
#[test]
fn test_get_library_path_for_plugin() {
    let pl = unsafe {PluginLoader::get_instance()};
    println!("Waiting for lock");
    let mut pl = pl.lock().unwrap();
    println!("Got lock get_library_path_for_plugin");
    let out = pl.get_library_path_for_plugin(CString::new("vamp-example-plugins:fixedtempo").unwrap());
    println!("{:#?}", out);
}
