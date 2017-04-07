use std::ffi::CString;
pub struct RealTime {
    sec: i32,
    nsec: i32
}

pub enum CxxFeature {}
pub struct Feature {
    timestamp: Option<RealTime>,
    duration: Option<RealTime>,
    values: Vec<f32>,
    label: CString
}
