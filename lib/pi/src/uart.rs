use core::fmt;
use core::time::Duration;

use shim::io;

use volatile::prelude::*;
use volatile::{ReadVolatile, Volatile};

use crate::common::{CLOCK_HZ, IO_BASE};
use crate::gpio::{Function, Gpio};
use crate::timer;

/// The base address for the `MU` registers.
const MU_REG_BASE: usize = IO_BASE + 0x215040;

/// The `AUXENB` register from page 9 of the BCM2837 documentation.
const AUX_ENABLES: *mut Volatile<u8> = (IO_BASE + 0x215004) as *mut Volatile<u8>;

/// The target baud rate.
const BAUD_RATE: u64 = 115200;

/// The value that needs to be written to the AUX_MU_BAUD register to
/// achieve the target baud rate. Obtained by solving the equation on
/// page 11 of the BCM2837 peripherals manual for baudrate_reg. This
/// also doesn't get the exact baud rate because it's rounded but UART
/// appears to be forgiving enough for that.
const BAUD_REG_VAL: u64 = CLOCK_HZ / (8 * BAUD_RATE) - 1;

/// Enum representing bit fields of the `AUX_MU_LSR_REG` register.
#[repr(u8)]
enum LsrStatus {
    DataReady = 1,
    TxAvailable = 1 << 5,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    IO_REG: Volatile<u32>,
    IER_REG: Volatile<u32>,
    IIR_REG: Volatile<u32>,
    LCR_REG: Volatile<u32>,
    MCR_REG: Volatile<u32>,
    LSR_REG: ReadVolatile<u32>,
    MSR_REG: ReadVolatile<u32>,
    SCRATCH: Volatile<u32>,
    CNTL_REG: Volatile<u32>,
    STAT_REG: ReadVolatile<u32>,
    BAUD: Volatile<u32>,
}

/// The Raspberry Pi's "mini UART".
pub struct MiniUart {
    registers: &'static mut Registers,
    timeout: Option<Duration>,
}

impl MiniUart {
    /// Initializes the mini UART by enabling it as an auxiliary peripheral,
    /// setting the data size to 8 bits, setting the BAUD rate to ~115200 (baud
    /// divider of 270), setting GPIO pins 14 and 15 to alternative function 5
    /// (TXD1/RDXD1), and finally enabling the UART transmitter and receiver.
    ///
    /// By default, reads will never time out. To set a read timeout, use
    /// `set_read_timeout()`.
    pub fn new() -> MiniUart {
        let registers = unsafe {
            // Enable the mini UART as an auxiliary device.
            (*AUX_ENABLES).or_mask(1);
            &mut *(MU_REG_BASE as *mut Registers)
        };

        // set baud rate to 115200(ish)
        registers.BAUD.write(BAUD_REG_VAL as u32);

        // initialize gpio pins to correct states (alt5 for 14/15)
        Gpio::new(14).into_alt(Function::Alt5);
        Gpio::new(15).into_alt(Function::Alt5);

        // enable UART transmitter/receiver
        registers.CNTL_REG.write(0b11);

        MiniUart {
            registers,
            timeout: None,
        }
    }

    /// Set the read timeout to `t` duration.
    pub fn set_read_timeout(&mut self, t: Duration) {
        self.timeout = Some(t);
    }

    /// Write the byte `byte`. This method blocks until there is space available
    /// in the output FIFO.
    pub fn write_byte(&mut self, byte: u8) {
        // block until transmit FIFO has space
        while !self
            .registers
            .LSR_REG
            .has_mask(LsrStatus::TxAvailable as u32)
        {}

        self.registers.IO_REG.write(byte as u32);
    }

    /// Returns `true` if there is at least one byte ready to be read. If this
    /// method returns `true`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately. This method does not block.
    pub fn has_byte(&self) -> bool {
        self.registers.LSR_REG.has_mask(LsrStatus::DataReady as u32)
    }

    /// Blocks until there is a byte ready to read. If a read timeout is set,
    /// this method blocks for at most that amount of time. Otherwise, this
    /// method blocks indefinitely until there is a byte to read.
    ///
    /// Returns `Ok(())` if a byte is ready to read. Returns `Err(())` if the
    /// timeout expired while waiting for a byte to be ready. If this method
    /// returns `Ok(())`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately.
    pub fn wait_for_byte(&self) -> Result<(), ()> {
        // block until a byte is received or the timeout finishes, whichever comes first
        if let Some(timeout) = self.timeout {
            let mut wait_timer = timer::Timer::new();
            wait_timer.set_compare_reg(1, timeout);

            while !self.has_byte() && !wait_timer.compare_triggered(1) {}
        } else {
            // just block on receiving a byte if there's no timeout configured
            while !self.has_byte() {}
        }

        if self.has_byte() {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Reads a byte. Blocks indefinitely until a byte is ready to be read.
    pub fn read_byte(&mut self) -> u8 {
        while !self.has_byte() {}
        self.registers.IO_REG.read() as u8
    }
}

impl fmt::Write for MiniUart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            // write a carriage return before any newline
            if byte == b'\n' {
                self.write_byte(b'\r');
            }

            self.write_byte(byte);
        }

        Ok(())
    }
}

mod uart_io {
    use super::io;
    use super::MiniUart;
    use shim::ioerr;
    use volatile::prelude::*;

    impl io::Read for MiniUart {
        fn read(&mut self, dst: &mut [u8]) -> io::Result<usize> {
            // wait for the first byte
            let wait_res = self.wait_for_byte();
            if wait_res.is_err() {
                // FIXME: TimedOut isn't available in acid_io :(
                return ioerr!(TimedOut, "UART timed out when waiting for first byte");
            }

            // read the rest of the bytes that are immediately available
            let mut idx = 0;
            while self.has_byte() && idx < dst.len() {
                dst[idx] = self.read_byte();
                idx += 1;
            }

            Ok(idx + 1)
        }
    }

    impl io::Write for MiniUart {
        fn write(&mut self, src: &[u8]) -> io::Result<usize> {
            for byte in src {
                // again, newlines must be preceded by carriage returns
                if *byte == b'\n' {
                    self.write_byte(b'\r');
                }

                self.write_byte(*byte);
            }

            Ok(src.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            // not sure if this is what they wanted? just waits until the transmit FIFO is completely empty
            while !self.registers.STAT_REG.has_mask(1 << 8) {}
            Ok(())
        }
    }

    impl io::Write for &mut MiniUart {
        fn write(&mut self, src: &[u8]) -> io::Result<usize> {
            (*self).write(src)
        }

        fn flush(&mut self) -> io::Result<()> {
            (*self).flush()
        }
    }
}
