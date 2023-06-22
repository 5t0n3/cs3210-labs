#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

use core::arch::asm;
use core::time::Duration;
use pi;
use xmodem::Xmodem;

/// Start address of the binary to load and of the bootloader.
const BINARY_START_ADDR: usize = 0x200000;

/// Pointer to where the loaded binary expects to be laoded.
const BINARY_START: *mut u8 = BINARY_START_ADDR as *mut u8;

/// Branches to the address `addr` unconditionally.
unsafe fn jump_to(addr: *mut u8) -> ! {
    asm!(
        "br {dest}",
        dest = in(reg) addr as usize,
        options(noreturn)
    )
}

unsafe fn kmain() -> ! {
    // FIXME: Implement the bootloader.

    let binary_len = 0;
    let s = core::slice::from_raw_parts_mut(BINARY_START, binary_len);
    jump_to(BINARY_START)
}
