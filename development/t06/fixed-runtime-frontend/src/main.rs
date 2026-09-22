#![forbid(unsafe_code)]
use std::path::Path;
fn main() {
    let Ok(args) = arguments() else {
        eprintln!("fixed frontend: invalid or overbound arguments");
        std::process::exit(1);
    };
    let result = if args.len() == 4 && args[1] == "run" {
        hee3_fixed_runtime_frontend::frontend::run(Path::new(&args[2]), Path::new(&args[3]), &args)
    } else if args.len() == 6 && args[1] == "inspect-recovery" {
        hee3_fixed_runtime_frontend::recovery::inspect(
            Path::new(&args[2]),
            &args[3],
            &args[4],
            Path::new(&args[5]),
        )
    } else if args.len() == 7 && args[1] == "inspect-recovery" {
        hee3_fixed_runtime_frontend::recovery::inspect_cursor(
            Path::new(&args[2]),
            &args[3],
            &args[4],
            Path::new(&args[5]),
            &args[6],
        )
    } else if args.len() == 9 && args[1] == "finish-cancelled" {
        hee3_fixed_runtime_frontend::recovery_cancel::finish(
            hee3_fixed_runtime_frontend::recovery_cancel::Request {
                root: Path::new(&args[2]),
                store_generation: &args[3],
                epoch: &args[4],
                task: &args[5],
                expected_generation: &args[6],
                event: &args[7],
            },
            Path::new(&args[8]),
        )
    } else {
        Err("usage: fixed frontend run <frozen manifest> <fresh absolute output> OR inspect-recovery <existing store root> <generation> <epoch> <fresh report directory> [complete cursor JSON] OR finish-cancelled <store root> <store generation> <epoch> <task> <expected generation> <event> <fresh report directory>".into())
    };
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn arguments() -> Result<Vec<String>, ()> {
    let mut values = Vec::new();
    for value in std::env::args_os().take(10) {
        let value = value.into_string().map_err(|_| ())?;
        if values.len() >= 9 || value.len() > 4096 {
            return Err(());
        }
        values.push(value);
    }
    Ok(values)
}
