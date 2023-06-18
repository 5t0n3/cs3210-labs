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
    let mut pin5 = Gpio::new(5).into_output();
    let mut pin6 = Gpio::new(6).into_output();
    let mut pin13 = Gpio::new(13).into_output();
    let mut pin16 = Gpio::new(16).into_output();
    let mut pin19 = Gpio::new(19).into_output();
    let mut pin26 = Gpio::new(26).into_output();
    let half_second = Duration::from_millis(500);

    loop {
        // turn on zig zag in order
        pin5.set();
        spin_sleep(half_second);
        pin6.set();
        spin_sleep(half_second);
        pin13.set();
        spin_sleep(half_second);
        pin16.set();
        spin_sleep(half_second);
        pin19.set();
        spin_sleep(half_second);
        pin26.set();
        spin_sleep(half_second);

        // do the same thing but turning off
        pin5.clear();
        spin_sleep(half_second);
        pin6.clear();
        spin_sleep(half_second);
        pin13.clear();
        spin_sleep(half_second);
        pin16.clear();
        spin_sleep(half_second);
        pin19.clear();
        spin_sleep(half_second);
        pin26.clear();
        spin_sleep(half_second);
    }
}
