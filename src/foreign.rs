use util::*;
use std::os::raw::{c_char, c_void};
use wren_sys;
use std::collections::HashMap;
use std::ffi::CString;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct MethodDesc {
    pub module: CString,
    pub class_name: CString,
    pub is_static: bool,
    pub signature: CString,
}

#[doc(hidden)]
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ClassDesc {
    pub module: CString,
    pub class_name: CString,
}

#[derive(Debug, Default)]
pub struct Foreign {
    pub classes: HashMap<ClassDesc, wren_sys::WrenForeignClassMethods>,
    pub methods: HashMap<MethodDesc, wren_sys::WrenForeignMethodFn>,
}

pub struct ForeignMethod {
    pub signature: &'static str,
    pub method: unsafe extern "C" fn(*mut ::wren_sys::WrenVM),
}

pub trait WrenClass: Default + Sized {
    const ID: usize;
    const MODULE: &'static str;
    const CLASS: &'static str;
}

impl WrenClass for () {
    const ID: usize = ::std::usize::MAX;
    const MODULE: &'static str = "<none>";
    const CLASS: &'static str = "Unit";
}

#[doc(hidden)]
#[repr(C)]
pub struct ForeignClass<T: WrenClass> {
    valid: bool,
    id: usize,
    data: T,
}

impl<T> ForeignClass<T>
where
    T: WrenClass,
{
    pub fn new(data: T) -> Self {
        ForeignClass {
            valid: true,
            data,
            id: T::ID,
        }
    }

    pub fn get_data(&self) -> &T {
        &self.data
    }

    pub fn get_data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub unsafe fn is_valid(ptr: *mut ForeignClass<T>) -> bool {
        (*ptr).valid
    }
}

impl<T> Default for ForeignClass<T>
where
    T: WrenClass,
{
    fn default() -> Self {
        ForeignClass {
            data: Default::default(),
            id: T::ID,
            valid: true,
        }
    }
}

impl Foreign {
    pub(crate) fn bind_class<T: WrenClass>(&mut self) {
        self.classes.insert(
            ClassDesc {
                module: from_str(T::MODULE),
                class_name: from_str(T::CLASS),
            },
            T::bind_foreign(),
        );
    }

    pub(crate) fn bind_method(
        &mut self,
        module: &str,
        class_name: &str,
        is_static: bool,
        name: &str,
        method: ForeignMethod,
    ) {
        self.methods.insert(
            MethodDesc {
                module: from_str(module),
                class_name: from_str(class_name),
                is_static: is_static,
                signature: from_str(&format!("{}{}", name, method.signature)),
            },
            Some(method.method),
        );
    }
}

trait ToWren {
    fn bind_foreign() -> wren_sys::WrenForeignClassMethods;
}

impl<T> ToWren for T
where
    T: WrenClass,
{
    fn bind_foreign() -> wren_sys::WrenForeignClassMethods {
        wren_sys::WrenForeignClassMethods {
            allocate: Some(alloc_foreign_class::<T>),
            finalize: Some(finalize_foreign_class::<T>),
        }
    }
}

pub(crate) unsafe extern "C" fn alloc_foreign_class<T>(vm: *mut wren_sys::WrenVM)
where
    T: WrenClass,
{
    use std::mem::{forget, size_of, swap};
    let mut v: ForeignClass<T> = Default::default();
    wren_sys::wrenEnsureSlots(vm, 1);
    let p = wren_sys::wrenSetSlotNewForeign(vm, 0, 0, size_of::<ForeignClass<T>>()) as
        *mut ForeignClass<T>;
    swap(&mut v, &mut *p);
    forget(v);
}

pub(crate) unsafe extern "C" fn alloc_invalid_class(vm: *mut wren_sys::WrenVM) {
    use std::mem::{forget, size_of, swap};
    let mut data = ForeignClass::<()>::default();
    data.valid = false;
    wren_sys::wrenEnsureSlots(vm, 1);
    let p = wren_sys::wrenSetSlotNewForeign(vm, 0, 0, size_of::<ForeignClass<()>>()) as
        *mut ForeignClass<()>;
    swap(&mut data, &mut *p);
    forget(data);
}

pub(crate) unsafe extern "C" fn finalize_foreign_class<T: WrenClass>(ptr: *mut c_void) {
    use std::mem::{swap, uninitialized};
    let mut v: ForeignClass<T> = uninitialized();
    swap(&mut v, &mut *(ptr as *mut ForeignClass<T>));
    if v.id != T::ID {
        panic!("invalid cast of foreign class in finalizer")
    }
}

#[allow(non_snake_case)]
pub(crate) unsafe extern "C" fn bind_foreign_class(
    vm: *mut wren_sys::WrenVM,
    module: *const c_char,
    className: *const c_char,
) -> wren_sys::WrenForeignClassMethods {
    let user_data = wren_sys::wrenGetUserData(vm) as *const ::vm::UserData;
    let foreigns = &(*user_data).foreigns;
    let desc = ClassDesc {
        module: c_string(module),
        class_name: c_string(className),
    };
    match foreigns.classes.get(&desc) {
        Some(binding) => *binding,
        None => wren_sys::WrenForeignClassMethods {
            allocate: Some(alloc_invalid_class),
            finalize: Some(finalize_foreign_class::<()>),
        },
    }
}

#[allow(non_snake_case)]
pub(crate) unsafe extern "C" fn bind_foreign_method(
    vm: *mut wren_sys::WrenVM,
    module: *const c_char,
    className: *const c_char,
    isStatic: bool,
    signature: *const c_char,
) -> wren_sys::WrenForeignMethodFn {
    let user_data = wren_sys::wrenGetUserData(vm) as *const ::vm::UserData;
    let foreigns = &(*user_data).foreigns;
    let desc = MethodDesc {
        module: c_string(module),
        class_name: c_string(className),
        is_static: isStatic,
        signature: c_string(signature),
    };
    match foreigns.methods.get(&desc) {
        Some(binding) => *binding,
        None => None,
    }
}
