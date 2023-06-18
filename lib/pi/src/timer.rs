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

    /// Sets the provided comparison register to trigger after the provided duration.
    /// Also clears the corresponding bit in the CS register.
    // TODO: dynamically select register based on which ones are in use already?
    pub fn set_compare_reg(&mut self, reg_num: usize, wait: Duration) {
        if reg_num >= 4 {
            panic!("Timer::set_compare_reg(): provided register too large")
        }

        let cs_mask = 1 << reg_num;

        // set specified compare register
        let current_time = self.read();
        let compare_micros = (current_time + wait).as_micros();
        self.registers.COMPARE[reg_num].write(0);
        self.registers.COMPARE[reg_num].write(compare_micros as u32);

        // write a 1 bit to clear the corresponding stuff in the CS register/interrupts/?
        self.registers.CS.write(cs_mask);
    }

    /// Checks whether the provided system timer compare register has been hit since
    /// the corresponding bit in the CS register was last cleared.
    pub fn compare_triggered(&self, reg_num: usize) -> bool {
        if reg_num >= 4 {
            panic!("Timer::compare_triggered(): provided register too large")
        }

        self.registers.CS.has_mask(1 << reg_num)
    }
}

/// Returns current time.
pub fn current_time() -> Duration {
    Timer::new().read()
}

/// Spins until `t` duration have passed.
pub fn spin_sleep(t: Duration) {
    let mut timer = Timer::new();
    timer.set_compare_reg(0, t);

    // do a busy loop until the CS register updates once compare target is hit
    'spin: loop {
        if timer.compare_triggered(0) {
            break 'spin;
        }
    }
}
