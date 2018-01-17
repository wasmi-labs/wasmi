#![feature(link_args)]
#![feature(lang_items)]
#![feature(start)]
#![no_std]

#[lang="panic_fmt"]
extern fn panic_fmt(_: ::core::fmt::Arguments, _: &'static str, _: u32) -> ! {
}

#[lang = "eh_personality"]
extern fn eh_personality() {
}

#[link_args = "-s EXPORTED_FUNCTIONS=['_hello_world']"]
extern {}

#[no_mangle]
pub fn hello_world() -> isize {
    45 + 99
}

#[start]
fn main(argc: isize, argv: *const *const u8) -> isize {
    /* Intentionally left blank */
    0
}
