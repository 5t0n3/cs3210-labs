mod parsers;

use serial;
use structopt;
use xmodem::{Progress, Xmodem};

use std::path::PathBuf;
use std::time::Duration;

use serial::core::{BaudRate, CharSize, FlowControl, SerialDevice, SerialPortSettings, StopBits};
use structopt::StructOpt;

use parsers::{parse_baud_rate, parse_flow_control, parse_stop_bits, parse_width};

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(
        short = "i",
        help = "Input file (defaults to stdin if not set)",
        parse(from_os_str)
    )]
    input: Option<PathBuf>,

    #[structopt(
        short = "b",
        long = "baud",
        parse(try_from_str = parse_baud_rate),
        help = "Set baud rate",
        default_value = "115200"
    )]
    baud_rate: BaudRate,

    #[structopt(
        short = "t",
        long = "timeout",
        parse(try_from_str),
        help = "Set timeout in seconds",
        default_value = "10"
    )]
    timeout: u64,

    #[structopt(
        short = "w",
        long = "width",
        parse(try_from_str = parse_width),
        help = "Set data character width in bits",
        default_value = "8"
    )]
    char_width: CharSize,

    #[structopt(help = "Path to TTY device", parse(from_os_str))]
    tty_path: PathBuf,

    #[structopt(
        short = "f",
        long = "flow-control",
        parse(try_from_str = parse_flow_control),
        help = "Enable flow control ('hardware' or 'software')",
        default_value = "none"
    )]
    flow_control: FlowControl,

    #[structopt(
        short = "s",
        long = "stop-bits",
        parse(try_from_str = parse_stop_bits),
        help = "Set number of stop bits",
        default_value = "1"
    )]
    stop_bits: StopBits,

    #[structopt(short = "r", long = "raw", help = "Disable XMODEM")]
    raw: bool,
}

fn main() {
    use std::fs::File;
    use std::io::{self, BufRead, BufReader};

    let opt = Opt::from_args();
    let mut port = serial::open(&opt.tty_path).expect("path points to invalid TTY");
    port.set_timeout(Duration::from_secs(opt.timeout))
        .expect("set port timeout");

    // grab existing settings and modify them as specified
    let mut settings = port
        .read_settings()
        .expect("couldn't read serial port settings");
    settings
        .set_baud_rate(opt.baud_rate)
        .expect("error when setting baud rate");
    settings.set_char_size(opt.char_width);
    settings.set_flow_control(opt.flow_control);
    settings.set_stop_bits(opt.stop_bits);

    port.write_settings(&settings)
        .expect("error when updating port settings");

    // yay polymorphism :)
    let mut reader: Box<dyn BufRead> = if let Some(path) = opt.input {
        let handle = File::open(path).expect("couldn't open file");
        Box::new(BufReader::new(handle))
    } else {
        Box::new(io::stdin().lock())
    };

    let bytes_written = if opt.raw {
        io::copy(&mut reader, &mut port).expect("writing raw data") as usize
    } else {
        Xmodem::transmit_with_progress(reader, port, progress_fn).expect("writing with xmodem")
    };

    println!(
        "Successfully wrote {} bytes to your destination!",
        bytes_written
    );
}

fn progress_fn(progress: Progress) {
    match progress {
        Progress::Waiting => println!("Waiting for receiver NAK..."),
        Progress::Started => println!("Transfer started!"),
        Progress::Packet(n) => println!("Sending packet {}", n),
        _ => {}
    };
}
