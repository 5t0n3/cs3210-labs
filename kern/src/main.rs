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

// FIXME: You need to add dependencies here to
// test your drivers (Phase 2). Add them as needed.

#[allow(dead_code)]
fn kmain() -> ! {
    // FIXME: Start the shell.
    loop {
        shell::shell("> ");
    }
}
