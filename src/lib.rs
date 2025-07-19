use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

pub mod parse_ini;
pub use parse_ini::{parse_ini, parse_ini_with_config, IniParserConfig};

mod ffi {
    #![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

extern "C" fn rust_handler(
    user: *mut c_void,
    section: *const c_char,
    name: *const c_char,
    value: *const c_char,
) -> i32 {
    unsafe {
        let section = CStr::from_ptr(section).to_string_lossy().into_owned();
        let name = CStr::from_ptr(name).to_string_lossy().into_owned();
        let value = CStr::from_ptr(value).to_string_lossy().into_owned();

        let map = &mut *(user as *mut HashMap<String, HashMap<String, String>>);
        map.entry(section)
            .or_default()
            .insert(name, value);
    }
    1 // continue parsing
}

pub fn parse_ini_file(filename: &str) -> Option<HashMap<String, HashMap<String, String>>> {
    let c_filename = CString::new(filename).ok()?;
    let mut data: HashMap<String, HashMap<String, String>> = HashMap::new();

    let result = unsafe {
        let data_ptr = &mut data as *mut _ as *mut c_void;
        ffi::ini_parse(c_filename.as_ptr(), Some(rust_handler), data_ptr)
    };

    if result == 0 {
        Some(data)
    } else {
        None
    }
}

