use shim::io;
use shim::path::{Path, PathBuf};

use stack_vec::StackVec;

use pi::atags::Atags;

use fat32::traits::FileSystem;
use fat32::traits::{Dir, Entry};

use crate::console::{kprint, kprintln, CONSOLE};
use crate::ALLOCATOR;
use crate::FILESYSTEM;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

fn execute_command(c: Command) {
    match c.path() {
        "echo" => {
            for (n, arg) in c.args.iter().enumerate() {
                // don't print the path/command
                if n != 0 {
                    kprint!("{} ", arg);
                }
            }
            kprintln!();
        }
        unknown => {
            kprintln!("unknown command: {}", unknown);
        }
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns.
pub fn shell(prefix: &str) -> ! {
    let mut console = CONSOLE.lock();
    let mut current_line = [0; 512];
    let mut line_vec = StackVec::new(&mut current_line);
    loop {
        kprint!("{}", prefix);
        'command: loop {
            match console.read_byte() {
                // backspace or delete
                8 | 127 => {
                    if line_vec.len() > 0 {
                        // backspace, overwrite with space & backspace again
                        console.write_byte(8);
                        console.write_byte(b' ');
                        console.write_byte(8);
                        line_vec.pop();
                    }
                }

                // newline
                b'\r' | b'\n' => {
                    kprintln!();
                    let mut args = [""; 64];

                    match Command::parse(
                        core::str::from_utf8(line_vec.as_slice()).unwrap(),
                        &mut args,
                    ) {
                        Ok(command) => execute_command(command),
                        Err(Error::TooManyArgs) => kprintln!("error: too many arguments"),
                        _ => (),
                    };
                    line_vec.truncate(0);
                    break 'command;
                }

                // other control characters (print bell character)
                c if c.is_ascii_control() => kprint!("\u{7}"),

                // normal characters
                c => {
                    if let Ok(()) = line_vec.push(c) {
                        kprint!("{}", c as char);
                    }
                }
            }
        }
    }
}
