//extern crate bindgen;
//use std::env;
//use std::path::PathBuf;
extern crate cpp_build;

fn main() {
    ////////////// BINDGEN ////////////////
    // Tell cargo to tell rustc to link with the system's vamp library
    println!("cargo:rustc-link-lib=vamp-hostsdk");
    
//    let bindings = bindgen::Builder::default()
//        //.no_unstable_rust()
//        //.enable_cxx_namespaces()
//        .raw_line("pub use self::root::*;")
//        .header("wrapper.hpp")
//        .clang_arg("-x")
//        .clang_arg("c++")
//        .clang_arg("-stdlib=libc++")
//        .whitelist_recursively(false)
//        .whitelisted_type("*PluginLoader")
//        .whitelisted_type("*Plugin")
//        .link("vamp-hostsdk")
//        .generate()
//        .expect("Unabled to generate bindings");

//    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
//    bindings.write_to_file(out_path.join("bindings.rs"))
//        .expect("Couldn't write bindings");
    /////////////// BINDGEN ////////////////

    ////////////// RUST-CPP ////////////////
    cpp_build::Config::new().flag("-std=c++11").build("src/lib.rs");
}
