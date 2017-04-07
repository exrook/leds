use std::ffi::CString;
pub enum SampleType {
    OneSamplePerStep,
    FixedSampleRate(f32),
    VariableSampleRate(f32)
}
pub enum CxxOutputDescriptor {}
pub struct OutputDescriptor {
    pub identifier: CString,
    pub name: CString,
    pub description: Option<CString>,
    pub unit: CString,
    /// Present if there is a fixed bin size, if zero, output is point data
    pub bin_count: Option<usize>,
    pub bin_names: Option<Vec<CString>>,
    /// (Min,Max) possible range of values if present
    pub extents: Option<(f32,f32)>,
    /// If present, resolution values are quantized to
    pub quantizeStep: Option<f32>,
    pub sample_type: SampleType,
    pub has_duration: bool,
}
