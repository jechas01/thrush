#[macro_use]
extern crate thrush;

use std::collections::HashMap;

use thrush::vm::*;
use thrush::foreign::{ForeignMethod, WrenClass};

#[derive(Default, Clone)]
struct MyMap(HashMap<String, String>);

impl WrenClass for MyMap {
    const ID: usize = 0;
    const MODULE: &'static str = "main";
    const CLASS: &'static str = "RustMap";
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

use std::fmt::Write;

const COMBINE: ForeignMethod = wren_fn!(vm, (
    map: [MyMap]
) -> String {
    let map = unsafe { &*map };
    let mut out = String::new();
    for (key, value) in map.0.iter() {
        write!(out, "{}: {}\n", key, value);
    }
    out
});

const GET: ForeignMethod = wren_fn!(vm, Brackets, (
    map: [MyMap],
    key: String
) -> String {
    let map = unsafe { &*map };
    map.0.get(&key).map(Clone::clone).unwrap_or_default()
});

const CONTAINS: ForeignMethod = wren_fn!(vm, (
    map: [MyMap],
    key: String
) -> bool {
    let map = unsafe { &*map };
    map.0.contains_key(&key)
});

const COPY: ForeignMethod = wren_fn!(vm, (
    map: [MyMap]
) -> [MyMap] {
    let map = unsafe { &*map };
    map.clone()
});

const UHOH: ForeignMethod = wren_fn!(vm, (
    map: [MyMap]
) -> [LolWut] {
    LolWut
});

#[derive(Default)]
struct LolWut;

impl WrenClass for LolWut {
    const ID: usize = 3;
    const MODULE: &'static str = "main";
    const CLASS: &'static str = "LolWut";
}

const SCRIPT: &'static str = r##"
foreign class RustMap {
    construct new() {}
    foreign insert(key,value)
    foreign combine()
    foreign contains(key)
    foreign [key]
    foreign clone()
    foreign wut()
    print() {
        System.print(combine())
    }
}

var map = RustMap.new()

map.insert("Hello", "World")
map.insert("foo", "bar")

var bar = map.clone()

map.insert("spam", "eggs")

if (map.contains("spam")) {
    System.print(map["spam"])
}

System.print(bar.contains("spam"))

map.print()
map.wut()
"##;

#[test]
fn foreigns() {
    let mut vm = WrenBuilder::new()
        .bind_class::<MyMap>()
        .bind_method("main", "RustMap", false, "insert", INSERT)
        .bind_method("main", "RustMap", false, "", GET)
        .bind_method("main", "RustMap", false, "contains", CONTAINS)
        .bind_method("main", "RustMap", false, "combine", COMBINE)
        .bind_method("main", "RustMap", false, "clone", COPY)
        .bind_method("main", "RustMap", false, "wut", UHOH)
        .build();
    vm.interpret(SCRIPT).unwrap();
    panic!()
}
