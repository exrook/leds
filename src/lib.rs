#![feature(trace_macros)]
#[link(name = "vamp-hostsdk")]
extern {}
#[macro_use]
extern crate cpp;
#[macro_use]
extern crate cpp_macros;

#[cfg(test)]
mod tests; 
macro_rules! cppp {
    ($fn_name:ident $data_name:ident $data_ptr_name:ident ($cxx_obj_name:ident : $cxx_obj_type:expr => $cxx_obj_ptr:expr) [$(let $p_name:ident : $p_type:ty = $var:expr ),+] ($($c_name:ident : $c_type:ty),*) $body:block) => {
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
        //cpp!([mut
    }
}

macro_rules! c_rustfn {
    ($fn_name:ident $data_name:ident $data_ptr_name:ident [$(let $p_name:ident : $p_type:ty = $var:expr ),+] ($($c_name:ident : $c_type:ty),*) $body:block) => {
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

pub mod host_ext;
pub mod plugin;
pub mod cxx_util;
