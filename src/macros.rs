#[macro_export]
macro_rules! wren_fn {
    ([[ $vm:expr ]] $block:block) => (
        (|| $block)()
    );
    ([[ $vm:expr, $slot:ident ]] bind_var $var_name:ident : f64) => (
        let $var_name: f64 = unsafe { $crate::sys::wrenGetSlotNum($vm, $slot) };
    );
    ([[ $vm:expr, $slot:ident ]] bind_var $var_name:ident : bool) => (
        let $var_name: bool = unsafe { $crate::sys::wrenGetSlotBool($vm, $slot) };
    );
    ([[ $vm:expr, $slot:ident ]] bind_var $var_name:ident : String) => (
        let $var_name: String = unsafe {
            let ptr = $crate::sys::wrenGetSlotString($vm, $slot);
            ::std::ffi::CStr::from_ptr(ptr).to_string_lossy().into()
        };
    );
    ([[ $vm:expr, $slot:ident ]] bind_var $var_name:ident : [$ty:ty]) => (
        let $var_name: *mut $ty = unsafe {
            let ptr = $crate::sys::wrenGetSlotForeign($vm, $slot) as *mut $crate::foreign::ForeignClass<$ty>;
            if ! $crate::foreign::ForeignClass::<$ty>::is_valid(ptr) {
                wren_fn!([[ $vm ]] abort "attempt to use unbound foreign class");
            }
            if (*ptr).get_id() != <$ty as $crate::foreign::WrenClass>::WREN_ID {
                wren_fn!([[ $vm ]] abort format!("foreign object with invalid type id for {}", stringify!($ty)));
            }
            (*ptr).get_data_mut()
        };
    );
    ([[ $vm:expr, $slot:ident ]] ensure_slot bool) => (
        wren_fn!([[ $vm, $slot ]] ensure_slot bool WREN_TYPE_BOOL);
    );
    ([[ $vm:expr, $slot:ident ]] ensure_slot String) => (
        wren_fn!([[ $vm, $slot ]] ensure_slot String WREN_TYPE_STRING);
    );
    ([[ $vm:expr, $slot:ident ]] ensure_slot f64) => (
        wren_fn!([[ $vm, $slot ]] ensure_slot f64 WREN_TYPE_NUM);
    );
    ([[ $vm:expr, $slot:ident ]] ensure_slot $t:tt) => (
        wren_fn!([[ $vm, $slot ]] ensure_slot $t WREN_TYPE_FOREIGN);
    );
    ([[ $vm:expr, $slot:ident ]] ensure_slot $rust:tt $wren:tt) => (
        unsafe {
            let slot_type = $crate::sys::wrenGetSlotType($vm, $slot);
            use $crate::sys::WrenType::*;
            match slot_type {
                $wren => {}
                t => wren_fn!([[ $vm ]] abort format!("expecting {} for variable {}, got {:?}.", stringify!($rust), $slot, t)),
            }
        }
    );
    ([[ $vm:expr, $slot:ident ]] bind_vars ) => ();
    ([[ $vm:expr, $slot:ident ]] bind_vars _ $(, $($t:tt):+)*) => (
        $slot += 1;
        wren_fn!([[ $vm, $slot ]] bind_vars $($($t):+),*);
    );
    ([[ $vm:expr, $slot:ident ]] bind_vars $var_name:ident : $var_type:tt $(,$($t:tt):+)*) => (
        wren_fn!([[ $vm, $slot ]] ensure_slot $var_type);
        wren_fn!([[ $vm, $slot ]] bind_var $var_name: $var_type);
        $slot += 1;
        wren_fn!([[ $vm, $slot ]] bind_vars $($($t):+),*);
    );
    ([[ $vm:expr ]] ($($t:tt)*) $($rest:tt)*) => (
        let total_slots = unsafe { $crate::sys::wrenGetSlotCount($vm) };
        let num_vars = wren_fn!(count_vars $($t)*);
        if num_vars > (total_slots) {
            wren_fn!([[ $vm ]] abort 
                format!(
                    "invalid number of arguments. expecting {}, got {}.",
                    num_vars,
                    total_slots-1,
                )
            );
        }
        #[allow(unused_variables)]
        let mut current_slot = 0;
        wren_fn!([[ $vm, current_slot ]] bind_vars $($t)*);
        drop(current_slot);
        wren_fn!([[ $vm ]] $($rest)*);
    );
    (build_sig_args_rest) => ("");
    (build_sig_args_rest $($head:tt):+) => (
        "_"
    );
    (build_sig_args_rest $($head:tt):+ $(, $($tail:tt):+)+) => (
        concat!("_,", wren_fn!(build_sig_args_rest $($($tail):+),+))
    );
    (build_sig_args $($head:tt):+ ) => ("");
    (build_sig_args $($head:tt):+ $(, $($tail:tt):+)+) => (
        wren_fn!(build_sig_args_rest $($($tail):+),+)
    );
    (build_sig ($($t:tt)*) $($rest:tt)*) => (
        concat!('(', wren_fn!(build_sig_args $($t)*), ')')
    );
    (count_vars) => (
        0
    );
    (count_vars $($head:tt):+ $(, $($tail:tt):+)* ) => (
        1 + wren_fn!(count_vars $($($tail):+),*)
    );
    ([[ $vm:expr ]] abort $msg:expr) => ({
        let error_message = ::std::ffi::CString::new($msg).unwrap();
        #[allow(unused_unsafe)]
        unsafe {
            $crate::sys::wrenEnsureSlots($vm, 1);
            $crate::sys::wrenSetSlotString($vm, 0, error_message.as_ptr());
            $crate::sys::wrenAbortFiber($vm, 0);
        }
        return;
    });
    ($vm_name:ident,  $($t:tt)+) => (
        ForeignMethod {
            signature: wren_fn!(build_sig $($t)+),
            method: {
                extern fn ignoreme($vm_name: *mut $crate::sys::WrenVM) {
                    wren_fn!([[ $vm_name ]] $($t)*);
                }
                ignoreme
            }
        }
    );
}