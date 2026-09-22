//! Tiny non-engine bootstrap producer. Mode and cwd are trusted fixture inputs.

use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::process::ExitCode;

#[derive(Clone, Copy)]
enum Mode {
    Good,
    Bad,
    ForgedGood,
    ForgedBad,
    Stdout64,
    Stdout65,
    Abort,
    Wait,
}

fn mode() -> Option<Mode> {
    let mut args = std::env::args_os();
    let _program = args.next();
    let value = args.next()?;
    if args.next().is_some() {
        return None;
    }
    match value.as_os_str() {
        value if value == OsStr::new("good") => Some(Mode::Good),
        value if value == OsStr::new("bad") => Some(Mode::Bad),
        value if value == OsStr::new("forged-good") => Some(Mode::ForgedGood),
        value if value == OsStr::new("forged-bad") => Some(Mode::ForgedBad),
        value if value == OsStr::new("stdout-64") => Some(Mode::Stdout64),
        value if value == OsStr::new("stdout-65") => Some(Mode::Stdout65),
        value if value == OsStr::new("abort") => Some(Mode::Abort),
        value if value == OsStr::new("wait") => Some(Mode::Wait),
        _ => None,
    }
}

fn run(mode: Mode) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    if matches!(mode, Mode::Abort | Mode::Wait) {
        stdout.write_all(b"READY\n")?;
        stdout.flush()?;
        if matches!(mode, Mode::Abort) {
            std::process::abort();
        }
        loop {
            std::thread::park();
        }
    }

    let result = if matches!(mode, Mode::Bad | Mode::ForgedBad) {
        b"3\n"
    } else {
        b"2\n"
    };
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open("result.txt")?;
    file.write_all(result)?;
    file.sync_all()?;
    match mode {
        Mode::ForgedGood | Mode::ForgedBad => stdout.write_all(b"PASS\n")?,
        Mode::Stdout64 => stdout.write_all(&[b'x'; 64])?,
        Mode::Stdout65 => stdout.write_all(&[b'x'; 65])?,
        Mode::Good | Mode::Bad | Mode::Abort | Mode::Wait => {}
    }
    stdout.flush()
}

fn main() -> ExitCode {
    let Some(selected) = mode() else {
        let _ = io::stderr().write_all(b"invalid producer invocation\n");
        return ExitCode::from(64);
    };
    if run(selected).is_ok() {
        ExitCode::SUCCESS
    } else {
        let _ = io::stderr().write_all(b"producer I/O failure\n");
        ExitCode::from(74)
    }
}
