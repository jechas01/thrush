use std::any::Any;
use errors::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::os::raw::{c_char, c_int, c_void};
use errors::WrenError;
use std::ffi::{CStr, CString};
use std::mem;
use wren_sys::{self, WrenConfiguration, WrenErrorType, WrenInterpretResult, WrenVM, wrenFreeVM,
               wrenGetUserData, wrenInitConfiguration, wrenNewVM, wrenSetUserData};
use foreign::*;

pub struct WrenBuilder {
    inner: WrenConfiguration,
    foreigns: Foreign,
}

unsafe extern "C" fn error_callback(
    vm: *mut WrenVM,
    ty: WrenErrorType,
    module: *const c_char,
    line: c_int,
    message: *const c_char,
) {
    let error: Result<Trace, WrenError> = match ty {
        WrenErrorType::WREN_ERROR_STACK_TRACE => Ok(Trace::new(message, module, line)),
        _ => Err(WrenError::new(ty, module, line, message)),
    };
    let user_data = wrenGetUserData(vm) as *const UserData;
    ((*user_data).error_cb)(error);
}

unsafe extern "C" fn write_callback(vm: *mut WrenVM, text: *const c_char) {
    let output = CStr::from_ptr(text).to_string_lossy();
    print!("{}", output);
}

pub(crate) struct UserData {
    pub(crate) foreigns: Foreign,
    error_cb: Box<Fn(Result<Trace, WrenError>)>,
}

impl WrenBuilder {
    pub fn new() -> WrenBuilder {
        let inner = unsafe {
            let mut cfg = mem::zeroed();
            wrenInitConfiguration(&mut cfg as *mut WrenConfiguration);
            cfg
        };
        WrenBuilder {
            foreigns: Default::default(),
            inner,
        }
    }

    pub fn bind_class<T: Sized + Default + ::std::fmt::Debug>(
        mut self,
        module: &str,
        class_name: &str,
    ) -> Self {
        self.foreigns.bind_class::<T>(module, class_name);
        self
    }

    pub fn build(self) -> Wren {
        let mut inner = self.inner;
        let error = Rc::new(RefCell::new(None));
        let error_ref = error.clone();
        let error_cb = Box::new(move |err: Result<Trace, WrenError>| {
            let mut err_mut = error_ref.borrow_mut();
            #[allow(unused_variables)]
            match err {
                Ok(trace) => match *err_mut {
                    Some(WrenError::Runtime {
                        ref message,
                        ref mut stack,
                    }) => stack.push(trace),
                    _ => panic!("got a trace without a runtime error"),
                },
                Err(err) => *err_mut = Some(err),
            }
        });
        let user_data = Box::new(UserData {
            foreigns: self.foreigns,
            error_cb,
        });

        inner.errorFn = Some(error_callback);
        inner.writeFn = Some(write_callback);
        inner.bindForeignClassFn = Some(bind_foreign_class);

        let sys_vm = unsafe { wrenNewVM(&mut inner as *mut WrenConfiguration) };
        unsafe { wrenSetUserData(sys_vm, Box::into_raw(user_data) as *mut c_void) };

        let wren = Wren {
            inner: sys_vm,
            error: error,
        };

        wren
    }
}

pub struct Wren {
    inner: *mut WrenVM,
    error: Rc<RefCell<Option<WrenError>>>,
}

impl Wren {
    pub fn interpret<S: Into<Vec<u8>>>(&mut self, source: S) -> Result<(), WrenError> {
        let c_source = CString::new(source).unwrap();
        let res = unsafe { ::wren_sys::wrenInterpret(self.inner, c_source.as_ptr()) };
        match res {
            WrenInterpretResult::WREN_RESULT_SUCCESS => Ok(()),
            _ => Err(self.error.borrow_mut().take().unwrap()),
        }
    }
}

impl Drop for Wren {
    fn drop(&mut self) {
        unsafe {
            let user_data = wrenGetUserData(self.inner) as *mut UserData;
            Box::from_raw(user_data);
            wrenFreeVM(self.inner);
        }
    }
}

#[test]
fn stuff() {
    {
        use std::collections::HashMap;
        ::env_logger::init().unwrap();
        let mut vm = WrenBuilder::new()
            .bind_class::<usize>("main", "Bar")
            .bind_class::<HashMap<String, String>>("main", "Foo")
            .build();
        vm.interpret(
            r#"
foreign class Foo {
    construct new() {}
}

foreign class Bar {
    construct new() {}
}

var bar = Bar.new()
var foo = Foo.new()
    "#,
        ).unwrap();
    }
    panic!();
}
