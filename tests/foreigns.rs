#[macro_use]
extern crate thrush;

use std::collections::HashMap;

use thrush::vm::*;
use thrush::foreign::{ForeignMethod, WrenClass};

#[derive(Default)]
struct MyMap(HashMap<String, String>);

impl WrenClass for MyMap {
    const WREN_ID: usize = 0;
}

const INSERT: ForeignMethod = wren_fn!(vm, (
    map: [MyMap],
    key: String,
    value: String
) {
    let map = unsafe { &mut *map };
    map.0.insert(key, value);
});


const PRINT: ForeignMethod = wren_fn!(vm, (
    map: [MyMap]
) {
    let map = unsafe { &*map };
    for (key, value) in map.0.iter() {
        println!("{}: {}", key, value);
    }
});

const SCRIPT: &'static str = r##"

foreign class RustMap {
    construct new() {}
    foreign insert(key,value)
    foreign print()
}

var map = RustMap.new()
map.insert("Hello", "World!")
map.insert("foo", "bar")
map.print()
"##;

#[test]
fn foreigns() {
    let mut vm = WrenBuilder::new()
        .bind_class::<MyMap>("main", "RustMap")
        .bind_method("main", "RustMap", false, "insert", INSERT)
        .bind_method("main", "RustMap", false, "print", PRINT)
        .build();
    vm.interpret(SCRIPT).unwrap();
}
