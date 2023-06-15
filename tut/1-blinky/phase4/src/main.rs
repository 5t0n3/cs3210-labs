#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

use core::arch::asm;

const GPIO_BASE: usize = 0x3F000000 + 0x200000;

const GPIO_FSEL1: *mut u32 = (GPIO_BASE + 0x04) as *mut u32;
const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;

#[inline(never)]
fn spin_sleep_ms(ms: usize) {
    for _ in 0..(ms * 6000) {
        unsafe {
            asm!("nop");
        }
    }
}

unsafe fn kmain() -> ! {
    // set GPIO pin 16 to output mode
    GPIO_FSEL1.write_volatile(0b001 << 18);

    loop {
        // set GPIO pin 16
        GPIO_SET0.write_volatile(1 << 16);
        spin_sleep_ms(500);

        // set GPIO pin 16 to off (clear it I guess?)
        GPIO_CLR0.write_volatile(1 << 16);
        spin_sleep_ms(500);
    }
}
