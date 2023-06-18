#![feature(alloc_error_handler)]
#![feature(decl_macro)]
#![feature(negative_impls)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

pub mod console;
pub mod mutex;
pub mod shell;

use console::kprintln;

// FIXME: You need to add dependencies here to
// test your drivers (Phase 2). Add them as needed.
use core::writeln;
use pi::uart::MiniUart;
use shim::io::*;

#[allow(dead_code)]
fn kmain() -> ! {
    // FIXME: Start the shell.
    let mut uart = MiniUart::new();
    loop {
        let byte = uart.read_byte();
        writeln!(
            uart,
            "You typed: {}",
            char::from_u32(byte as u32).unwrap().escape_debug()
        )
        .unwrap();
    }
}
