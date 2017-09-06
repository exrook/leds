extern crate cpp_build;

fn main() {
    cpp_build::Config::new().flag("-std=c++11").build("src/lib.rs");
}
