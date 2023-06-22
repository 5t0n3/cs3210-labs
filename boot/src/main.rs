#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

use core::arch::asm;
use core::fmt::Write;
use core::time::Duration;
use core::write;
use pi;
use shim::io;
use xmodem::Xmodem;

/// Start address of the binary to load and of the bootloader.
const BINARY_START_ADDR: usize = 0x200000;

/// Pointer to where the loaded binary expects to be laoded.
const BINARY_START: *mut u8 = BINARY_START_ADDR as *mut u8;

/// Arbitrary maximum size of the kernel binary (64 MB)
const KERNEL_MAX_SIZE: usize = 64 * 1000 * 1000;

/// Branches to the address `addr` unconditionally.
unsafe fn jump_to(addr: *mut u8) -> ! {
    asm!(
        "br {dest}",
        dest = in(reg) addr as usize,
        options(noreturn)
    )
}

unsafe fn kmain() -> ! {
    // initialize mini UART on the pi
    let mut uart = pi::uart::MiniUart::new();
    uart.set_read_timeout(Duration::from_millis(750));

    // create a slice in memory for the kernel to be placed into
    let kernel_buf = core::slice::from_raw_parts_mut(BINARY_START, KERNEL_MAX_SIZE);

    loop {
        // attempt to read the kernel into memory as transmitted over UART
        match Xmodem::receive(&mut uart, &mut *kernel_buf) {
            // actually run the kernel code if it's read successfully
            Ok(_) => jump_to(BINARY_START),

            // silently attempt again on timeout
            Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,

            // print an error message if something else went wrong
            Err(e) => write!(uart, "error bootstrapping kernel: {}", e).unwrap(),
        }
    }
}
