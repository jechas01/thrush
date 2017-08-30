use std::os::raw::{c_char, c_int};
use std::ffi::{CStr, CString};
use wren_sys::WrenErrorType;

#[derive(Debug)]
pub struct Trace {
    pub function: String,
    pub module: String,
    pub line: u32,
}

unsafe fn lossy_string(input: *const c_char) -> String {
    CStr::from_ptr(input).to_string_lossy().into_owned()
}

impl Trace {
    pub(crate) unsafe fn new(function: *const c_char, module: *const c_char, line: c_int) -> Trace {
        Trace {
            function: CStr::from_ptr(function).to_string_lossy().into_owned(),
            module: CStr::from_ptr(module).to_string_lossy().into_owned(),
            line: line as u32,
        }
    }
}

#[derive(Debug)]
pub enum WrenError {
    Compile {
        module: String,
        line: u32,
        message: String,
    },
    Runtime { message: String, stack: Vec<Trace> },
}

impl WrenError {
    pub(crate) unsafe fn new(
        ty: WrenErrorType,
        module: *const c_char,
        line: c_int,
        message: *const c_char,
    ) -> WrenError {
        match ty {
            WrenErrorType::WREN_ERROR_COMPILE => WrenError::Compile {
                module: lossy_string(module),
                line: line as u32,
                message: lossy_string(message),
            },
            _ => WrenError::Runtime {
                message: lossy_string(message),
                stack: vec![],
            },
        }
    }
}
