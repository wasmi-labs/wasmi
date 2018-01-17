#![feature(link_args)]
#![feature(lang_items)]
#![feature(start)]
#![no_std]
#![no_main]

#[lang="panic_fmt"]
extern fn panic_fmt(_: ::core::fmt::Arguments, _: &'static str, _: u32) {
}

#[lang = "eh_personality"]
extern fn eh_personality() {
}

#[link_args = "-s WASM=1 -s NO_EXIT_RUNTIME=1 -s NO_FILESYSTEM=1"]
extern {}

#[no_mangle]
pub fn hello_world() -> isize {
    45 + 99
}