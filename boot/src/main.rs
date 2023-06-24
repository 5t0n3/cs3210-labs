#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

use core::arch::asm;
use core::fmt::Write;
use core::time::Duration;
use core::writeln;

use pi::uart::MiniUart;
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

fn kmain() -> ! {
    // initialize mini UART on the pi with a 750ms timeout
    let mut uart = MiniUart::new();
    uart.set_read_timeout(Duration::from_millis(750));

    // create a slice in memory for the kernel to be placed into
    // this is safe because there is literally nothing else that could be using the
    // memory at this point :)
    let kernel_buf: &mut [u8] =
        unsafe { core::slice::from_raw_parts_mut(BINARY_START, KERNEL_MAX_SIZE) };

    loop {
        // attempt to read the kernel into memory as transmitted using XMODEM over UART
        match Xmodem::receive(&mut uart, &mut *kernel_buf) {
            // actually run the kernel code if it's read successfully
            // this is technically always unsafe but we can't avoid it
            Ok(_) => unsafe { jump_to(BINARY_START) },

            // silently attempt again on timeout
            Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,

            // print an error message if something else went wrong
            // TODO: figure out how to like view the error messages? screen locks ttys which is annoying
            Err(e) => {
                writeln!(&mut uart, "error bootstrapping kernel: {}", e).unwrap();
            }
        }
    }
}
