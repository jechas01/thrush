use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub unsafe fn c_string(input: *const c_char) -> CString {
    CStr::from_ptr(input).to_owned()
}

pub fn from_str(input: &str) -> CString {
    CString::new(input).unwrap()
}

pub unsafe fn lossy_string(input: *const c_char) -> String {
    CStr::from_ptr(input).to_string_lossy().into()
}
