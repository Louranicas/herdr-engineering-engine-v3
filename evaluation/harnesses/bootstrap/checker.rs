//! Independent fixed-literal checker; no engine imports or producer verdicts.

use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Clone, Copy)]
enum Mode {
    Compare,
    Abort,
}

fn invocation() -> Option<(Mode, PathBuf)> {
    let mut args = std::env::args_os();
    let _program = args.next();
    let selected = args.next()?;
    let input = PathBuf::from(args.next()?);
    if args.next().is_some() {
        return None;
    }
    let mode = if selected == OsStr::new("compare") {
        Mode::Compare
    } else if selected == OsStr::new("abort") {
        Mode::Abort
    } else {
        return None;
    };
    Some((mode, input))
}

fn captured_bytes(path: &Path) -> io::Result<Vec<u8>> {
    // The coordinator owns an immutable capture. These checks reject ordinary
    // type mistakes; they do not claim protection against hostile path races.
    let before = fs::symlink_metadata(path)?;
    if !before.is_file() || before.file_type().is_symlink() {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    let file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    let mut bytes = Vec::with_capacity(3);
    file.take(3).read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn run(mode: Mode, input: &Path) -> io::Result<()> {
    let bytes = captured_bytes(input)?;
    let mut stdout = io::stdout().lock();
    if matches!(mode, Mode::Abort) {
        stdout.write_all(b"READY\n")?;
        stdout.flush()?;
        std::process::abort();
    }
    if bytes == b"2\n" {
        stdout.write_all(b"MATCH\n")?;
    } else {
        stdout.write_all(b"MISMATCH\n")?;
    }
    stdout.flush()
}

fn main() -> ExitCode {
    let Some((mode, input)) = invocation() else {
        let _ = io::stderr().write_all(b"invalid checker invocation\n");
        return ExitCode::from(64);
    };
    if run(mode, &input).is_ok() {
        ExitCode::SUCCESS
    } else {
        let _ = io::stderr().write_all(b"checker I/O failure\n");
        ExitCode::from(74)
    }
}
