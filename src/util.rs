use std::ffi::CStr;
use std::os::raw::c_char;
pub unsafe fn lossy_string(input: *const c_char) -> String {
    CStr::from_ptr(input).to_string_lossy().into_owned()
}
