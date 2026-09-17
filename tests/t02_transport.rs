//! Actual fake-child pipes for the T02 decoder/session boundary, under TH-DEV.
//! The default mode runs only five finite controls. Intentionally nonterminating
//! modes are invoked solely by `development/transport_process.py`'s supervisor.

use habitat_engine::contracts::{Generation, UuidV4};
use habitat_engine::worker::pi::{
    Binding, Command, Frame, Framer, Observation, Refusal, RunState, Session, validate_record,
};
use serde_json::{Value, json};
use std::io::{self, BufRead, Read, Write};
use std::process::{Command as ProcessCommand, Stdio};
use std::thread;
use std::time::Duration;

const UNICODE: &[u8] = include_bytes!("fixtures/pi/records/unicode-delta.jsonl");
const STATE: &[u8] = include_bytes!("fixtures/pi/records/state-empty.jsonl");
const COALESCED: &[u8] = b"{\"type\":\"agent_start\"}\n{\"type\":\"agent_settled\"}\n";
const TRUNCATED: &[u8] = b"{\"type\":\"agent_start\"}";
const WARNING: &[u8] = b"warning: HEE3_T02_FAKE_STDERR_CONTROL\n";
const FINITE_CASES: [&str; 5] = [
    "fragmented-lf-utf8",
    "coalesced-records",
    "stderr-empty-benign",
    "stderr-warning-fault",
    "truncated-eof",
];

fn session() -> Session<'static> {
    Session::new(Binding {
        task: UuidV4::parse("123e4567-e89b-42d3-a456-000000000001").unwrap(),
        attempt: UuidV4::parse("123e4567-e89b-42d3-a456-000000000002").unwrap(),
        generation: "7".parse::<Generation>().unwrap(),
    })
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        result.push(char::from(DIGITS[usize::from(byte >> 4)]));
        result.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    result
}

fn state_response(id: &Value) -> Vec<u8> {
    let mut value: Value = serde_json::from_slice(STATE).unwrap();
    value["id"] = id.clone();
    let mut bytes = serde_json::to_vec(&value).unwrap();
    bytes.push(b'\n');
    bytes
}

fn fake_worker(case: &str) -> io::Result<()> {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    if case == "timeout-fault" {
        output.write_all(TRUNCATED)?;
        output.flush()?;
        loop {
            thread::sleep(Duration::from_secs(60));
        }
    }
    if matches!(case, "stdout-flood-fault" | "stderr-flood-fault") {
        let mut destination: Box<dyn Write> = if case == "stdout-flood-fault" {
            Box::new(output)
        } else {
            Box::new(io::stderr().lock())
        };
        let chunk = [b'x'; 4096];
        loop {
            destination.write_all(&chunk)?;
            destination.flush()?;
        }
    }
    if case == "timeout-benign" {
        output.write_all(COALESCED)?;
        return output.flush();
    }
    if matches!(case, "stdout-flood-benign" | "stderr-flood-benign") {
        let chunk = [b'x'; 8192];
        if case == "stdout-flood-benign" {
            output.write_all(&chunk)?;
            return output.flush();
        }
        return io::stderr().write_all(&chunk);
    }

    let mut request = String::new();
    input.read_line(&mut request)?;
    let request: Value = serde_json::from_str(&request).unwrap();
    assert_eq!(request["type"], "get_state");
    assert!(request["id"].is_string());
    let response = state_response(&request["id"]);
    let bytes = match case {
        "fragmented-lf-utf8" => UNICODE,
        "coalesced-records" => COALESCED,
        "stderr-empty-benign" | "stderr-warning-fault" => &response,
        "truncated-eof" => TRUNCATED,
        _ => panic!("unknown fake-worker mode"),
    };
    if case == "fragmented-lf-utf8" {
        // Acknowledgement prevents the writer from racing ahead/coalescing writes.
        for &byte in bytes {
            output.write_all(&[byte])?;
            output.flush()?;
            let mut acknowledgement = [0];
            input.read_exact(&mut acknowledgement)?;
            assert_eq!(acknowledgement, [b'.']);
        }
    } else {
        output.write_all(bytes)?;
        output.flush()?;
    }
    if case == "stderr-warning-fault" {
        io::stderr().write_all(WARNING)?;
    }
    Ok(())
}

fn assert_outcome(
    case: &str,
    session: &mut Session<'_>,
    records: &[Frame],
    stdout: &[u8],
    stderr: &[u8],
    finish: Result<(), Refusal>,
) -> Option<&'static str> {
    let mut refusal = None;
    match case {
        "fragmented-lf-utf8" => {
            assert_eq!(stdout, UNICODE);
            assert!(stderr.is_empty());
            assert_eq!(finish, Ok(()));
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].bytes(), &UNICODE[..UNICODE.len() - 1]);
            validate_record(&records[0]).unwrap();
        }
        "coalesced-records" => {
            assert_eq!(stdout, COALESCED);
            assert!(stderr.is_empty());
            assert_eq!(finish, Ok(()));
            assert_eq!(records.len(), 2);
            assert_eq!(records[0].bytes(), b"{\"type\":\"agent_start\"}");
            assert_eq!(records[1].bytes(), b"{\"type\":\"agent_settled\"}");
            for frame in records {
                validate_record(frame).unwrap();
            }
        }
        "stderr-empty-benign" => {
            assert!(stderr.is_empty());
            assert_eq!(finish, Ok(()));
            assert_eq!(records.len(), 1);
            assert!(matches!(
                session.observe(&records[0], 1),
                Ok(Observation::State(_))
            ));
            assert_eq!(session.session_id(), Some("fixture-session-001"));
            assert_eq!(session.run_state(), RunState::Idle);
            assert!(session.issue(Command::State, 2).is_ok());
        }
        "stderr-warning-fault" => {
            assert_eq!(stderr, WARNING);
            assert_eq!(finish, Ok(()));
            assert_eq!(records.len(), 1);
            validate_record(&records[0]).unwrap();
            // The test process owner refuses the diagnostic before dispatching
            // even valid stdout. This is not an implemented production owner.
            session.transport_failed();
            refusal = Some("stderr");
        }
        "truncated-eof" => {
            assert_eq!(stdout, TRUNCATED);
            assert!(stderr.is_empty());
            assert!(records.is_empty());
            assert_eq!(finish, Err(Refusal::Truncated));
            session.transport_failed();
            refusal = Some("truncated-eof");
        }
        _ => unreachable!(),
    }
    refusal
}

fn finite_case(case: &str) -> io::Result<Value> {
    assert!(FINITE_CASES.contains(&case));
    let mut session = session();
    let request = session.issue(Command::State, 0).unwrap();
    let pending_id = session.pending_id().unwrap().to_owned();
    let mut child = ProcessCommand::new(std::env::current_exe()?)
        .args(["--fake-worker", case])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let child_pid = child.id();
    let mut input = child.stdin.take().unwrap();
    let mut output = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    // A separate drain preserves the diagnostic channel and avoids pipe blockage.
    let errors = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.take(65_537).read_to_end(&mut bytes).unwrap();
        assert!(bytes.len() <= 65_536, "finite fixture diagnostic overflow");
        bytes
    });
    input.write_all(&request)?;
    input.flush()?;
    let mut framer = Framer::default();
    let mut records = Vec::new();
    let mut stdout = Vec::new();
    let mut read_sizes = Vec::new();
    if case == "fragmented-lf-utf8" {
        for index in 0..UNICODE.len() {
            let mut byte = [0];
            output.read_exact(&mut byte)?;
            stdout.extend(byte);
            read_sizes.push(1);
            records.extend(framer.push(&byte).unwrap());
            assert_eq!(records.len(), usize::from(index + 1 == UNICODE.len()));
            input.write_all(b".")?;
            input.flush()?;
        }
    } else if case == "coalesced-records" {
        let mut bytes = vec![0; COALESCED.len()];
        output.read_exact(&mut bytes)?;
        read_sizes.push(bytes.len());
        records.extend(framer.push(&bytes).unwrap());
        stdout.extend(bytes);
    }
    drop(input);
    let mut buffer = [0; 4096];
    loop {
        let count = output.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        assert!(
            stdout.len() + count <= 65_536,
            "finite fixture stdout overflow"
        );
        read_sizes.push(count);
        stdout.extend_from_slice(&buffer[..count]);
        records.extend(framer.push(&buffer[..count]).unwrap());
    }
    let stderr = errors.join().unwrap();
    let exit = child.wait()?;
    assert!(exit.success(), "fake child failed: {exit}");
    let finish = framer.finish();
    if case == "truncated-eof" {
        assert_eq!(framer.push(b"\n").unwrap_err(), Refusal::Poisoned);
    }
    let refusal = assert_outcome(case, &mut session, &records, &stdout, &stderr, finish);
    if refusal.is_some() {
        assert_eq!(session.run_state(), RunState::Refused);
        assert_eq!(session.pending_id(), Some(pending_id.as_str()));
        assert_eq!(session.issue(Command::State, 2), Err(Refusal::Poisoned));
    }
    Ok(json!({
        "case": case, "status": "pass", "scope": "TH-DEV fake process",
        "child_pid": child_pid, "child_exit_code": exit.code(),
        "request_hex": hex(&request), "worker_stdout_hex": hex(&stdout),
        "worker_stderr_hex": hex(&stderr), "read_sizes": read_sizes,
        "records": records.len(), "transport_refusal": refusal,
        "module_admission": false,
    }))
}

fn supervised_case(case: &str) -> io::Result<()> {
    assert!(matches!(
        case,
        "timeout-fault"
            | "timeout-benign"
            | "stdout-flood-fault"
            | "stdout-flood-benign"
            | "stderr-flood-fault"
            | "stderr-flood-benign"
    ));
    // Both this waiting parent and its actual fake child are in the bounded
    // Python supervisor's process group. Raw child streams reach its real pipes.
    let mut child = ProcessCommand::new(std::env::current_exe()?)
        .args(["--fake-worker", case])
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let status = child.wait()?;
    assert!(status.success(), "supervised fixture exited: {status}");
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [mode, case] if mode == "--fake-worker" => fake_worker(case),
        [mode, case] if mode == "--supervised-case" => supervised_case(case),
        [mode, case] if mode == "--case" => {
            println!("{}", finite_case(case)?);
            Ok(())
        }
        [] => {
            for case in FINITE_CASES {
                println!("{}", finite_case(case)?);
            }
            println!("T02_TRANSPORT_FINITE_CONTROLS: 5 passed; TH-DEV only");
            Ok(())
        }
        _ => panic!("unsupported transport test arguments"),
    }
}
