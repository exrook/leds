use std::ffi::{CStr,CString};
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ops::Index;

use ::plugin::{CxxFeature,Feature};
pub enum CxxString { }

#[test]
fn test_cxxstring() {
    println!("Testing CxxString");
    let s = CString::new("Yolo").unwrap();
    let string = CxxString::new(&s);
    println!("String: {:#?}", string.to_c_string());
}

impl CxxString {
    pub fn new(s: &CStr) -> Box<CxxString> {
        let s_ptr = s.as_ptr();
        let string = unsafe{ cpp!([s_ptr as "char*"] -> *mut CxxString as "std::string*" {
            return new std::string(s_ptr);
        })};
        not_null!(string);
        return unsafe{ Box::from_raw(string) };
    }
    pub fn to_c_string(&self) -> CString {
        let string = self as *const _;
        let c_str = unsafe{CStr::from_ptr(cpp!([string as "std::string*"] -> *const c_char as "const char*" {
            return string->c_str();
        }))};
        return CString::from(c_str);
    }
    pub unsafe fn delete(&mut self) {
        let stringg = self as *mut _;
        cpp!( [stringg as "std::string*"] {
            delete stringg;
        });
    }
}

impl Drop for CxxString {
    fn drop(&mut self) {
        println!("Dropping CxxString");
        let string = self as *mut _;
        unsafe{cpp!([string as "std::string*"] {
            delete string;
        })};
        println!("Dropped CxxString");
    }
}

pub enum CxxInnerVector {}
pub struct CxxVector<'a, T: 'a> {
    _marker: PhantomData<&'a T>,
    _inner: Box<CxxInnerVector>,
}

impl<'a,T> CxxVector<'a,T> {
    pub unsafe fn from(ptr: *mut CxxInnerVector) -> Self {
        return Self {
            _marker: PhantomData,
            _inner: Box::from_raw(ptr),
        }
    }
    pub fn into_raw(self) -> *mut CxxInnerVector {
        return Box::into_raw(self._inner);
    }
}

impl<'a> CxxVector<'a,CxxString> {
    pub fn size(&self) -> usize {
        let vector = & (*(self._inner)) as *const _;
        unsafe {cpp!([vector as "std::vector<std::string>*"] -> usize as "size_t" {
            return vector->size();
        })}
    }
    pub fn to_vec(&self) -> Vec<CString> {
        let len = self.size();
        let mut vec = Vec::new();
        for i in 0..len {
            vec.push(self[i].to_c_string());
        }
        vec
    }
    pub unsafe fn delete(self) {
        let cxxvec = self.into_raw();
        unsafe {cpp!([cxxvec as "std::vector<std::string>*"] {
            delete cxxvec;
        })};
    }
}

impl<'a> Index<usize> for CxxVector<'a,CxxString> {
    type Output = CxxString;
    fn index(&self, index: usize) -> &Self::Output {
        let vector = & (*(self._inner)) as *const _;
        let ptr = unsafe {cpp!([vector as "std::vector<std::string>*", index as "size_t"] -> *const CxxString as "const std::string*" {
            return &(*vector)[index];
        })};
        not_null!(ptr);
        return unsafe {& *ptr};
    }
}
impl<'a> CxxVector<'a,CxxFeature> {
    pub fn size(&self) -> usize {
        let vector = & (*(self._inner)) as *const _;
        unsafe {cpp!([vector as "std::vector<Vamp::Plugin::Feature>*"] -> usize as "size_t" {
            return vector->size();
        })}
    }
    pub fn to_vec(&self) -> Vec<Feature> {
        let len = self.size();
        let mut vec = Vec::new();
        for i in 0..len {
            vec.push(self[i].to_rust());
        }
        vec
    }
    pub unsafe fn delete(self) {
        let cxxvec = self.into_raw();
        unsafe {cpp!([cxxvec as "std::vector<Vamp::Plugin::Feature>*"] {
            delete cxxvec;
        })};
    }
}

impl<'a> Index<usize> for CxxVector<'a,CxxFeature> {
    type Output = CxxFeature;
    fn index(&self, index: usize) -> &Self::Output {
        let vector = & (*(self._inner)) as *const _;
        let ptr = unsafe {cpp!([vector as "std::vector<Vamp::Plugin::Feature>*", index as "size_t"] -> *const CxxFeature as "const Vamp::Plugin::Feature*" {
            return &(*vector)[index];
        })};
        not_null!(ptr);
        return unsafe {& *ptr};
    }
}
impl<'a> CxxVector<'a,f32> {
    pub fn size(&self) -> usize {
        let vector = & (*(self._inner)) as *const _;
        unsafe {cpp!([vector as "std::vector<float>*"] -> usize as "size_t" {
            return vector->size();
        })}
    }
    pub fn to_vec(&self) -> Vec<f32> {
        let len = self.size();
        let mut vec = Vec::new();
        for i in 0..len {
            vec.push(self[i]);
        }
        vec
    }
    pub unsafe fn delete(self) {
        let cxxvec = self.into_raw();
        unsafe {cpp!([cxxvec as "std::vector<float>*"] {
            delete cxxvec;
        })};
    }
}

impl<'a> Index<usize> for CxxVector<'a,f32> {
    type Output = f32;
    fn index(&self, index: usize) -> &Self::Output {
        let vector = & (*(self._inner)) as *const _;
        let ptr = unsafe {cpp!([vector as "std::vector<float>*", index as "size_t"] -> *const f32 as "const float*" {
            return &(*vector)[index];
        })};
        not_null!(ptr);
        return unsafe {& *ptr};
    }
}
