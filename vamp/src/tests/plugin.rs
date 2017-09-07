use std::ffi::CString;
use ::{Plugin,InputDomain,PluginLoader,RealTime};

fn get_plugin() -> Box<Plugin> {
    let pl = unsafe {PluginLoader::get_instance()};
    let mut pl = pl.lock().unwrap();
    pl.load_plugin(CString::new("vamp-example-plugins:fixedtempo").unwrap(), 44100.0, 0x03).unwrap()
}

#[test]
fn test_initialize() {
    let mut pl = get_plugin();
    pl.initialise(1,24,256).unwrap();
}

#[test]
fn test_reset() {
    let mut pl = get_plugin();
    pl.initialise(1,24,256).unwrap();
    pl.reset();
}

#[test]
fn test_get_input_domain() {
    let pl = get_plugin();
    let dom = pl.get_input_domain();
    match dom {
        InputDomain::TimeDomain => { }
        _ => {panic!("InputDomain did not match")}
    }
}

#[test]
fn test_get_preferred_block_size() {
    let pl = get_plugin();
    let size = pl.get_preferred_block_size();
    println!("Block size: {:#?}", size);
    if size != 256 {
        panic!("Block size did not match");
    }
}

#[test]
fn test_get_preferred_step_size() {
    let pl = get_plugin();
    let size = pl.get_preferred_step_size();
    println!("Step size: {:#?}", size);
    if size != 64 {
        panic!("Step size did not match");
    }
}

#[test]
fn test_get_min_channel_count() {
    let pl = get_plugin();
    let count = pl.get_min_channel_count();
    println!("Min channel count: {:#?}", count);
    if count != 1 {
        panic!("Min channel count did not match");
    }
}

#[test]
fn test_get_max_channel_count() {
    let pl = get_plugin();
    let count = pl.get_max_channel_count();
    println!("Max channel count: {:#?}", count);
    if count != 1 {
        panic!("Max channel count did not match");
    }
}

#[test]
fn test_get_output_descriptors() {
    let pl = get_plugin();
    let descriptors = pl.get_output_descriptors();
    println!("Output descriptors: {:#?}", descriptors);
}

#[test]
fn test_process() {
    let mut pl = get_plugin();
    pl.initialise(1,24,256).unwrap();
    for i in 0..2000 {
        let channel = vec!(0.0; 256);
        let data = vec!(channel);
        let stamp = RealTime::new(0,0);
        let out = pl.process(data,stamp);
        println!("Features: {:#?}", out);
    }
    let out = pl.get_remaining_features();
    println!("Features: {:#?}", out);
}

#[test]
fn test_get_type() {
    let pl = get_plugin();
    let t = pl.get_type();
    println!("type: {:#?}", t);
    if t != CString::new("Feature Extraction Plugin").unwrap() {
        panic!("Type does not match");
    }
}

#[test]
fn test_get_vamp_api_version() {
    let pl = get_plugin();
    let ver = pl.get_vamp_api_version();
    println!("Vamp API version: {:#?}", ver);
    if ver != 2 {
        panic!("Vamp API version does not match");
    }
}

#[test]
fn test_get_identifier() {
    let pl = get_plugin();
    let ident = pl.get_identifier();
    println!("Identifier: {:#?}", ident);
    if ident != CString::new("fixedtempo").unwrap() {
        panic!("Identifier does not match");
    }
}

#[test]
fn test_get_name() {
    let pl = get_plugin();
    let name = pl.get_name();
    println!("Name: {:#?}", name);
    if name != CString::new("Simple Fixed Tempo Estimator").unwrap() {
        panic!("Name does not match");
    }
}

#[test]
fn test_get_description() {
    let pl = get_plugin();
    let description = pl.get_description();
    println!("Description: {:#?}", description);
    if description != CString::new("Study a short section of audio and estimate its tempo, assuming the tempo is constant").unwrap() {
        panic!("Description does not match");
    }
}

#[test]
fn test_get_maker() {
    let pl = get_plugin();
    let maker = pl.get_maker();
    println!("Maker: {:#?}", maker);
    if maker != CString::new("Vamp SDK Example Plugins").unwrap() {
        panic!("Maker does not match");
    }
}

#[test]
fn test_get_copyright() {
    let pl = get_plugin();
    let copyright = pl.get_copyright();
    println!("Copyright: {:#?}", copyright);
    if copyright != CString::new("Code copyright 2008 Queen Mary, University of London.  Freely redistributable (BSD license)").unwrap() {
        panic!("Copyright does not match");
    }
}

#[test]
fn test_get_version() {
    let pl = get_plugin();
    let version = pl.get_plugin_version();
    println!("Version: {:#?}", version);
    if version != 1 {
        panic!("Version does not match");
    }
}

#[test]
fn test_get_parameter_descriptors() {
    let pl = get_plugin();
    let descriptors = pl.get_parameter_descriptors();
    println!("Parameter descriptors: {:#?}", descriptors);
}

#[test]
fn test_get_parameter() {
    let pl = get_plugin();
    let param = pl.get_parameter(CString::new("minbpm").unwrap());
    println!("Parameter: {:#?}", param);
    if param != 50.0 {
        panic!("Parameter does not match");
    }
}

#[test]
fn test_set_parameter() {
    let mut pl = get_plugin();
    pl.set_parameter(CString::new("maxbpm").unwrap(), 20.0);
    let param = pl.get_parameter(CString::new("maxbpm").unwrap());
    println!("Set parameter: {:#?}", param);
    if param != 20.0 {
        panic!("Set parameter does not match");
    }
}

#[test]
fn test_get_programs() {
    let pl = get_plugin();
    let progs = pl.get_programs();
    println!("Programs: {:#?}", progs);
}

#[test]
fn test_get_current_program() {
    let pl = get_plugin();
    let prog = pl.get_current_program();
    println!("Programs: {:#?}", prog);
}

#[test]
fn test_select_program() {
    let mut pl = get_plugin();
    pl.select_program(CString::new("").unwrap());
}

