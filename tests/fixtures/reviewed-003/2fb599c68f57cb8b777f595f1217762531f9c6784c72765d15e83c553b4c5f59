//! Fixed public workload driver. No expected answers or acceptance authority.
//! A hostile linked library shares this process and can forge its output.

use std::fs::File;
use std::io::{self, Read, Write};
use std::process::ExitCode;
use strict_u64_workload::{ParseError, parse_u64};

const INPUT_PATH: &str = "/frozen/inputs.hex";
const COUNT: usize = 335;
const MAX_PUBLIC: u64 = 32 * 1024;
const MAX_INPUT: usize = 1025;

fn main() -> ExitCode {
    if run().is_ok() {
        ExitCode::SUCCESS
    } else {
        eprintln!("public wrapper input/output refused");
        ExitCode::from(2)
    }
}
fn run() -> Result<(), ()> {
    let file = File::open(INPUT_PATH).map_err(|_| ())?;
    let mut bytes = Vec::new();
    file.take(MAX_PUBLIC + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    let inputs = decode_inputs(&bytes)?;
    let stdout = io::stdout();
    let mut output = stdout.lock();
    for (index, input) in inputs.iter().enumerate() {
        match parse_u64(input) {
            Ok(value) => writeln!(output, "{index}\tok\t{value}"),
            Err(error) => writeln!(
                output,
                "{index}\terror\t{}",
                match error {
                    ParseError::Empty => "Empty",
                    ParseError::InvalidCharacter => "InvalidCharacter",
                    ParseError::LeadingZero => "LeadingZero",
                    ParseError::Overflow => "Overflow",
                }
            ),
        }
        .map_err(|_| ())?;
    }
    output.flush().map_err(|_| ())
}
fn decode_inputs(bytes: &[u8]) -> Result<Vec<String>, ()> {
    if u64::try_from(bytes.len()).map_err(|_| ())? > MAX_PUBLIC {
        return Err(());
    }
    let lines = bytes.strip_suffix(b"\n").ok_or(())?;
    let mut inputs = Vec::with_capacity(COUNT);
    for line in lines.split(|b| *b == b'\n') {
        if inputs.len() == COUNT || line.len() > MAX_INPUT * 2 || line.len() % 2 != 0 {
            return Err(());
        }
        let mut input = Vec::with_capacity(line.len() / 2);
        for pair in line.as_chunks::<2>().0 {
            input.push(nibble(pair[0])? * 16 + nibble(pair[1])?);
        }
        inputs.push(String::from_utf8(input).map_err(|_| ())?);
    }
    if inputs.len() != COUNT {
        return Err(());
    }
    Ok(inputs)
}
fn nibble(byte: u8) -> Result<u8, ()> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(()),
    }
}
