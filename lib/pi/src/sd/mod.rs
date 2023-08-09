use crate::common::{states, IO_BASE};
use core::marker::PhantomData;

use bilge::prelude::*;
use core::convert::TryFrom;

use volatile::prelude::*;
use volatile::Volatile;

mod commands;

use commands::{BusWidth, CMD8Arg, CommandError};

const EMMC_REG_BASE: usize = IO_BASE + 0x300000;

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    /// argument issued to ACMD23 (SET_WR_BLK_ERASE_COUNT)
    ARG2: Volatile<u32>,

    /// number/size of blocks to be transferred
    BLKSIZECNT: Volatile<u32>,

    /// argument to any command besides ACMD23 (see ARG2 above)
    ARG1: Volatile<u32>,

    /// register used to issue commands
    CMDTM: Volatile<u32>,

    /// response registers for commands.
    /// for responses spanning multiple registers (e.g. to CMD2, CMD9, CMD10),
    /// more significant bytes go in higher registers
    RESP0: Volatile<u32>,
    RESP1: Volatile<u32>,
    RESP2: Volatile<u32>,
    RESP3: Volatile<u32>,

    /// data read from/to be written to the SD card
    DATA: Volatile<u32>,

    /// debugging information about the card/EMMC controller
    /// the BCM2837 peripherals guide recommends using the INTERRUPT register over this
    STATUS: Volatile<u32>,

    /// EMMC module configuration registers
    CONTROL0: Volatile<u32>,
    CONTROL1: Volatile<u32>,

    /// contains interrupt flags obeying the mask stored in IRPT_MASK
    INTERRUPT: Volatile<u32>,

    /// mask for interrupts in INTERRUPT register
    IRPT_MASK: Volatile<u32>,

    /// enables interrupts on the int_to_arm output (not entirely sure what this means)
    IRPT_EN: Volatile<u32>,

    /// third EMMC module configuration register    
    CONTROL2: Volatile<u32>,

    /// allows for the faking of different interrupts for debugging purposes
    FORCE_IRPT: Volatile<u32>,

    /// number of clock cycles it takes for an EMMC card in boot mode to time out
    BOOT_TIMEOUT: Volatile<u32>,

    /// selects which submodules are accessible from the debug bus
    DBG_SEL: Volatile<u32>,

    /// allows for tuning of dma_req (?) threshold for data reads
    EXRDFIFO_CFG: Volatile<u32>,

    /// enables/bypasses the extension data register
    EXRDFIFO_EN: Volatile<u32>,

    /// card clock delay before sampling data/command responses
    TUNE_STEP: Volatile<u32>,

    /// number of steps to delay before sampling card response in SDR mode (?)
    TUNE_STEPS_STD: Volatile<u32>,

    /// card sample delay in DDR mode (?)
    TUNE_STEPS_DDR: Volatile<u32>,

    /// whether interrupts in SPI mode are independent of the card select line
    SPI_INT_SPT: Volatile<u32>,

    /// version information & slot interrupt status
    SLOTISR_VER: Volatile<u32>,
}

/// OCR (operating conditions register)
#[bitsize(32)]
#[derive(FromBits)]
struct OCR {
    power_status: bool,
    card_capacity_status: bool,
    uhs_ii_status: bool,
    reserved: u4,
    // 1.8V switch accepted
    s18a: bool,
    v3_5to3_6: bool,
    v3_4to3_5: bool,
    v3_3to3_4: bool,
    v3_2to3_3: bool,
    v3_1to3_2: bool,
    v3_0to3_1: bool,
    v2_9to3_0: bool,
    v2_8to2_9: bool,
    v2_7to2_8: bool,
    reserved: u7,
    // reserved for low voltage
    reserved: bool,
    reserved: u7,
}

pub enum SDError {
    Timeout,
    Command(CommandError),
}

type SDResult<S> = Result<EMMC<S>, SDError>;

impl From<CommandError> for SDError {
    fn from(value: CommandError) -> Self {
        Self::Command(value)
    }
}

// see page 35/53 of simplified spec (also page 28/46)
states! { Idle, Ready, Identification, Transfer, Data, Standby, Disconnect }

pub struct EMMC<State> {
    active_rca: Option<u32>,
    registers: &'static mut Registers,
    ocr_reg: OCR,
    _state: PhantomData<State>,
}

impl<T> EMMC<T> {
    #[inline(always)]
    fn transition<S>(self) -> EMMC<S> {
        EMMC {
            active_rca: self.active_rca,
            registers: self.registers,
            ocr_reg: self.ocr_reg,
            _state: PhantomData,
        }
    }
}

impl EMMC<Idle> {
    fn new() -> Result<EMMC<Idle>, SDError> {
        let mut emmc = EMMC {
            registers: unsafe { &mut *(EMMC_REG_BASE as *mut Registers) },
            active_rca: None,
            ocr_reg: OCR::from(0),
            _state: PhantomData,
        };

        // issue CMD0 to return all commands to idle state
        emmc.execute_command(commands::Command::CMD0)?;
        Ok(emmc)
    }

    fn voltage_inquiry(self) -> Result<EMMC<Ready>, SDError> {
        // TODO: make arg const? doesn't change at all
        self.execute_command(commands::Command::CMD8(CMD8Arg::new(
            // 2.7-3.6V (only non-reserved one)
            u4::new(0b0001),
            0b10101010,
        )))?;
        Ok(self.transition())
    }
}

impl EMMC<Ready> {
    // TODO: make SDResult alias generic
    fn ocr_inquiry(&mut self) -> Result<Self, SDError> {
        self.execute_command(commands::Command::ACMD41)?;

        self.ocr_reg = self.registers.RESP0.read().into();
        Ok(*self)
    }

    fn switch_signal_voltage(&mut self) -> Result<(), SDError> {
        self.execute_command(commands::Command::CMD11)?;
        Ok(())
    }

    fn send_cids(self) -> Result<EMMC<Identification>, SDError> {
        self.execute_command(commands::Command::CMD2)?;
        Ok(self.transition())
    }
}

impl EMMC<Identification> {
    fn get_rca(self) -> Result<EMMC<Standby>, SDError> {
        self.execute_command(commands::Command::CMD3)?;

        self.active_rca = (self.registers.RESP0.read() & 0xffff0000).into();

        Ok(self.transition())
    }
}

impl EMMC<Standby> {
    fn select_card(self) -> SDResult<Transfer> {
        // this will panic if the current RCA is zero (i.e. it wasn't updated after CMD3)
        self.execute_command(commands::Command::CMD7(self.active_rca.unwrap()))?;
        Ok(self.transition())
    }
}

impl EMMC<Transfer> {
    fn set_bus_width(&mut self, width: BusWidth) -> Result<(), SDError> {
        self.execute_command(commands::Command::ACMD6(width))?;
        Ok(())
    }
}

pub fn init() -> SDResult<Transfer> {
    // plan: (see page 17/35 of simplified spec for UHS-I init)
    // also see page 32/50 for more complete flow chart (?)

    // CMD0 (reset to idle)
    let emmc_ready: EMMC<Ready> = EMMC::new()?
        // CMD8 (inquire about supported voltages)
        .voltage_inquiry()?
        // ACMD41 (1.8V voltage req/OCR query)
        .ocr_inquiry()?;

    // CMD11 to voltage switch (conditional on ACMD41)
    // switch to 1.8V signalling voltage if possible
    if emmc_ready.ocr_reg.s18a() {
        emmc_ready.switch_signal_voltage()?;
    }

    // TODO: assert CCS bit is set?

    // CMD2 (all send CID)
    // result is discarded since we don't really care about it
    let emmc_ident: EMMC<Identification> = emmc_ready.send_cids()?;

    let emmc_transfer: EMMC<Transfer> = emmc_ident
        // CMD3 (publish RCA; store in active_rca field)
        .get_rca()?
        // CMD7 (select card)
        .select_card()?;

    // ACMD6 (set bus width to 4)
    emmc_transfer.set_bus_width(BusWidth::FourBits)?;

    // TODO: UHS-I init (what even is tuning)

    // TODO: read SCR register

    Ok(emmc_transfer)
}
