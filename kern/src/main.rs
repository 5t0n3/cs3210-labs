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
use pi::timer::spin_sleep;

const GPIO_BASE: usize = 0x3F000000 + 0x200000;

const GPIO_FSEL1: *mut u32 = (GPIO_BASE + 0x04) as *mut u32;
const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;

#[allow(dead_code)]
unsafe fn kmain() -> ! {
    // FIXME: Start the shell.

    // set GPIO pin 16 to output mode
    GPIO_FSEL1.write_volatile(0b001 << 18);
    let sleep_period = Duration::from_millis(5000);

    loop {
        // set GPIO pin 16
        GPIO_SET0.write_volatile(1 << 16);
        spin_sleep(sleep_period);

        // set GPIO pin 16 to off (clear it I guess?)
        GPIO_CLR0.write_volatile(1 << 16);
        spin_sleep(sleep_period);
    }
}
