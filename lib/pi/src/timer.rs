use crate::common::IO_BASE;
use core::time::Duration;

use volatile::prelude::*;
use volatile::{ReadVolatile, Volatile};

/// The base address for the ARM system timer registers.
const TIMER_REG_BASE: usize = IO_BASE + 0x3000;

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    CS: Volatile<u32>,
    CLO: ReadVolatile<u32>,
    CHI: ReadVolatile<u32>,
    COMPARE: [Volatile<u32>; 4],
}

/// The Raspberry Pi ARM system timer.
pub struct Timer {
    registers: &'static mut Registers,
}

impl Timer {
    /// Returns a new instance of `Timer`.
    pub fn new() -> Timer {
        Timer {
            registers: unsafe { &mut *(TIMER_REG_BASE as *mut Registers) },
        }
    }

    /// Reads the system timer's counter and returns Duration.
    /// `CLO` and `CHI` together can represent the number of elapsed microseconds.
    pub fn read(&self) -> Duration {
        let timer_micros: u64 =
            ((self.registers.CHI.read() as u64) << 32) | self.registers.CLO.read() as u64;

        Duration::from_micros(timer_micros)
    }
}

/// Returns current time.
pub fn current_time() -> Duration {
    Timer::new().read()
}

/// Spins until `t` duration have passed.
pub fn spin_sleep(t: Duration) {
    let timer = Timer::new();

    // everything is in microseconds :)
    let sleep_micro_lsb = (timer.read() + t).as_micros() as u32;

    // TODO: use a free register instead of just register 0
    timer.registers.CS.and_mask(0b1110);
    timer.registers.COMPARE[0].write(sleep_micro_lsb);

    // do a busy loop until the CS register updates once compare target is hit
    'spin: loop {
        if timer.registers.CS.has_mask(0b0001) {
            break 'spin;
        }
    }
}
