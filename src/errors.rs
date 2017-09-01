use std::os::raw::{c_char, c_int};
use wren_sys::WrenErrorType;
use util::*;

#[derive(Debug)]
pub struct Trace {
    pub function: String,
    pub module: String,
    pub line: u32,
}

impl Trace {
    pub(crate) unsafe fn new(function: *const c_char, module: *const c_char, line: c_int) -> Trace {
        Trace {
            function: lossy_string(function),
            module: lossy_string(module),
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
