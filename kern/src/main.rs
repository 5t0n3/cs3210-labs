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
use core::time::Duration;
use pi::gpio::Gpio;
use pi::timer::spin_sleep;

#[allow(dead_code)]
fn kmain() -> ! {
    // FIXME: Start the shell.

    // set GPIO pin 16 to output mode
    let mut pin16 = Gpio::new(16).into_output();
    let sleep_period = Duration::from_millis(5000);

    loop {
        // set GPIO pin 16
        pin16.set();
        spin_sleep(sleep_period);

        // set GPIO pin 16 to off (clear it I guess?)
        pin16.clear();
        spin_sleep(sleep_period);
    }
}
