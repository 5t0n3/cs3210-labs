use bilge::prelude::*;
use core::convert::TryFrom;

use volatile::prelude::*;
use volatile::Writeable;

use super::EMMC;

#[bitsize(2)]
#[derive(FromBits)]
enum CommandType {
    Normal,
    Suspend,
    Resume,
    Abort,
}

#[bitsize(2)]
#[derive(FromBits)]
enum ResponseType {
    None,
    Length136,
    Length48,
    Length48Busy,
}

#[bitsize(1)]
#[derive(FromBits)]
enum DataDirection {
    HostToCard,
    CardToHost,
}

#[bitsize(2)]
#[derive(TryFromBits)]
enum AutoCommand {
    None,
    CMD12,
    CMD23,
}

/// A command in the proper format to be written to the CMDTM register.
#[bitsize(32)]
#[derive(TryFromBits)]
struct CommandPayload {
    reserved: u2,
    index: u6,
    command_type: CommandType,
    is_data: bool,
    index_check: bool,
    crc_check: bool,
    reserved: bool,
    response_type: ResponseType,
    reserved: u10,
    multi_block: bool,
    data_direction: DataDirection,
    auto_command: AutoCommand,
    block_counter: bool,
    reserved: bool,
}

pub enum CommandError {
    NoActiveCard,
    Timeout,
}

// TODO: have command constants in separate submodule?

/// GO_IDLE_STATE - resets all connected cards to idle state
pub(crate) const CMD0S: Command = Command {
    payload: CommandPayload::new(
        u6::new(0),
        CommandType::Normal,
        false,
        true,
        true,
        ResponseType::None,
        false,
        DataDirection::HostToCard,
        AutoCommand::None,
        false,
    ),
    is_app: false,
};

/// SEND_IF_COND - sends interface condition/query (mostly voltage information)
pub(crate) const CMD8S: Command = Command {
    payload: CommandPayload::new(
        u6::new(8),
        CommandType::Normal,
        false,
        true,
        true,
        ResponseType::Length48,
        false,
        DataDirection::HostToCard,
        AutoCommand::None,
        false,
    ),
    is_app: false,
};

/// APP_CMD - indicates the next command is an application command
const CMD55S: Command = Command {
    payload: CommandPayload::new(
        u6::new(55),
        CommandType::Normal,
        false,
        true,
        true,
        ResponseType::Length48,
        false,
        DataDirection::HostToCard,
        AutoCommand::None,
        false,
    ),
    is_app: false,
};

#[bitsize(32)]
#[derive(FromBits, Clone, Copy)]
pub struct CMD8Arg {
    reserved: u20,
    // TODO: dedicated type
    voltage: u4,
    check_pattern: u8,
}

// TODO: look at page 65/83 of simplified spec for more info
#[bitsize(32)]
#[derive(FromBits)]
pub struct CMD6Arg {
    switch: bool,
    reserved: u7,
    reserved: u4,
    reserved: u4,
    power_limit: u4,
    drive_strength: u4,
    command_system: u4,
    access_mode: u4,
}

#[bitsize(2)]
#[derive(TryFromBits)]
pub enum BusWidth {
    OneBit = 0b00,
    FourBits = 0b10,
}

#[bitsize(32)]
#[derive(FromBits, Clone, Copy)]
pub struct ACMD41Arg {
    reserved: bool,
    hcs: bool,
    reserved: bool,
    xpc: bool,
    reserved: u3,
    s18r: bool,
    voltage_window: u9,
    reserved: u15,
}

const ACMD41_SDHC_INQ: ACMD41Arg = ACMD41Arg::new(
    // BCM2837 EMMC module supports SDHC/SDXC
    true,
    // I'm not using an SDXC card so this is irrelevant (?)
    false,
    // query switching to 1.8V signalling voltage for UHS-I use
    true,
    // I think any voltage is supported?
    u9::new(0b111111111),
);

#[non_exhaustive]
pub enum Command {
    /// GO_IDLE_STATE
    CMD0,
    /// ALL_SEND_CID
    CMD2,
    /// SEND_RELATIVE_ADDR
    CMD3,
    /// SWITCH_FUNC
    CMD6(CMD6Arg),
    /// SELECT/DESELECT_CARD
    CMD7(u32),
    /// SEND_IF_COND
    CMD8(CMD8Arg),
    /// VOLTAGE_SWITCH
    CMD11,
    /// READ_SINGLE_BLOCK
    CMD17(u32),
    /// SEND_TUNING_BLOCK
    CMD19,
    /// APP_CMD
    CMD55(u32),
    /// SET_BUS_WIDTH
    ACMD6(BusWidth),
    /// SD_SEND_OP_COND
    ACMD41,
}

impl Command {
    fn is_app(&self) -> bool {
        use Command::*;

        match *self {
            ACMD6(_) => true,
            ACMD41 => true,
            _ => false,
        }
    }

    fn arg(&self) -> u32 {
        use Command::*;

        match *self {
            CMD8(arg) => arg.into(),
            // TODO: ensure this conversion works like I expect it to
            ACMD6(width) => u2::from(width).into(),
            ACMD41 => ACMD41_SDHC_INQ.into(),

            // CMD0/2/3/11/19
            _ => 0,
        }
    }

    fn cmdtm(&self) -> u32 {
        0
    }
}

impl<T> EMMC<T> {
    // TODO: make associated function on EMMC?
    // TODO: somehow enforce that argument matches supplied command?
    pub(super) fn execute_command(&mut self, command: Command) -> Result<(), CommandError> {
        // send CMD55 to indicate next command is an application command
        if let Command::ACMD41 = command {
            // default to RCA 0 for ACMD41 (executed before new RCA is published using CMD3)
            // TODO: no response since no card selected -> no status?
            self.execute_command(Command::CMD55(0))?;
        } else if command.is_app() {
            // ensure a card is selected otherwise
            // TODO: enforce command executions with specific states?
            let addr = self.active_rca.ok_or(CommandError::NoActiveCard)?;
            self.execute_command(Command::CMD55(addr))?;
        }

        // actually execute the command
        self.registers.ARG1.write(command.arg());
        self.registers.CMDTM.write(command.cmdtm());

        // TODO: figure out why timeout exists for CMD0/55?

        // TODO: wait for proper amount of time/interrupt/status

        // TODO: bitfieldify
        // TODO: timeout! (probably check both err bit and bit 0)
        // wait for command to finish
        while self.registers.INTERRUPT.read() & 1 != 1 {}

        Ok(())
    }
}
