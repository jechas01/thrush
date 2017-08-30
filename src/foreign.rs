use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use wren_sys;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Hash)]
struct MethodDesc {
    pub module: String,
    pub class_name: String,
    pub is_static: bool,
    pub signature: String,
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct ClassDesc {
    pub module: String,
    pub class_name: String,
}

#[derive(Debug, Default)]
pub struct Foreign {
    classes: HashMap<ClassDesc, wren_sys::WrenForeignClassMethods>,
    methods: HashMap<MethodDesc, ()>,
}

impl Foreign {
    pub(crate) fn bind_class<T: Sized + Default + ::std::fmt::Debug>(
        &mut self,
        module: &str,
        class_name: &str,
    ) {
        self.classes.insert(
            ClassDesc {
                module: module.into(),
                class_name: class_name.into(),
            },
            T::bind_allocate(),
        );
    }
}

pub trait ToWren {
    fn bind_allocate() -> wren_sys::WrenForeignClassMethods;
}

impl<T> ToWren for T
where
    T: Sized + Default + ::std::fmt::Debug,
{
    fn bind_allocate() -> wren_sys::WrenForeignClassMethods {
        wren_sys::WrenForeignClassMethods {
            allocate: Some(alloc_foreign_class::<T>),
            finalize: Some(finalize_foreign_class::<T>),
        }
    }
}

pub unsafe extern "C" fn alloc_foreign_class<T>(vm: *mut wren_sys::WrenVM)
where
    T: Sized + Default + ::std::fmt::Debug,
{
    use std::mem::{forget, size_of, swap};
    let mut v: T = Default::default();
    let p = wren_sys::wrenSetSlotNewForeign(vm, 0, 0, size_of::<T>()) as *mut T;
    swap(&mut v, &mut *p);
    forget(v);
}

pub unsafe extern "C" fn finalize_foreign_class<T>(ptr: *mut c_void) {
    use std::mem::{swap, uninitialized};
    let mut v: T = uninitialized();
    swap(&mut v, &mut *(ptr as *mut T));
}

pub unsafe extern "C" fn bind_foreign_class(
    vm: *mut wren_sys::WrenVM,
    module: *const c_char,
    className: *const c_char,
) -> wren_sys::WrenForeignClassMethods {
    let user_data = wren_sys::wrenGetUserData(vm) as *const ::vm::UserData;
    let foreigns = &(*user_data).foreigns;
    let desc = ClassDesc {
        module: CStr::from_ptr(module).to_string_lossy().into(),
        class_name: CStr::from_ptr(className).to_string_lossy().into(),
    };
    match foreigns.classes.get(&desc) {
        Some(binding) => *binding,
        // TODO: Something other than usize that errors out on allocate
        None => <usize>::bind_allocate(),
    }
}
