#![forbid(unsafe_code)]
//! The candidate-visible launcher contains only the namespace shim, not the
//! collector, protected oracle or coordinator composition.

#[path = "../worker/namespace_shim.rs"]
mod namespace_shim;

use namespace_shim::NamespaceExec;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Ok(args) = arguments() else {
        eprintln!("namespace shim: invalid arguments");
        return ExitCode::from(127);
    };
    if args.len() < 4 || args[0] != "__namespace-exec" || args[2] != "--" {
        eprintln!("namespace shim: invalid invocation");
        return ExitCode::from(127);
    }
    let arguments: Vec<&str> = args[4..].iter().map(String::as_str).collect();
    namespace_shim::run(&NamespaceExec {
        executable: &args[3],
        arguments: &arguments,
        working_directory: &args[1],
    })
}

fn arguments() -> Result<Vec<String>, ()> {
    let mut values = Vec::new();
    let mut bytes = 0_usize;
    for argument in std::env::args_os().skip(1).take(257) {
        let argument = argument.into_string().map_err(|_| ())?;
        bytes = bytes.checked_add(argument.len()).ok_or(())?;
        if values.len() >= 256 || argument.len() > 4096 || bytes > 64 * 1024 {
            return Err(());
        }
        values.push(argument);
    }
    Ok(values)
}
