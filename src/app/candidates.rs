//! The native candidate source (B14a-4, design R18 round 2): an attempt's candidate from the pinned
//! local model. The prompt is rendered from the class's three candidate inputs and the previous
//! verification, the model runs under the full-file adapter profile (templated: the server applies
//! the pinned model's chat template — measured 2026-09-26, raw mode never finished), and its text is
//! the FULL replacement of the class's one editable path — or a refusal the class check records,
//! never an edit — or a provider failure that stops the task. The model's wall time is the
//! attempt's work: the runtime's window is passed through, nothing is created.
//!
//! Everything past the candidate (apply, the check, the receipt, acceptance) is the runtime's.
//! Precondition (R18 A9): the model must be resident at the adapter's context when `next` is
//! called; B14b's dispatcher establishes it — the identity readback refuses anything else by name.

use super::evidence::digest;
use super::runtime::{Ask, Candidate, CandidateSource, Previous, REFUSED_CANDIDATE_SCHEMA};
use crate::check::consistency::U64_EDITABLE;
use crate::store::VerificationVerdict;
use crate::worker::native::{self, AdapterProfile, ProviderState};
use crate::worker::process::PendingChild;
use crate::worker::{
    Capabilities, Feature, Finish, Invocation, MAX_PROMPT_BYTES, Request, Selection, Usage,
};
use std::time::Instant;

/// Why the source's text is not a candidate, by name — each recorded by the class check as a
/// refused candidate under [`Refusal::name`] (one path with a repair refusal, R18 A5).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// The model stopped at its output cap (`done_reason: length`): the file is not whole.
    Truncated,
    /// The model returned nothing, or whitespace only.
    Empty,
    /// The text holds one fence line, or three or more: not one file.
    NotOneFile,
    /// The provider proposed tools; the class has none.
    Tools,
    /// The class prompt could not be rendered for this attempt (the site named).
    Prompt(&'static str),
}

impl Refusal {
    /// The reason the refused-candidate record carries, in the `candidate_*` family the repair
    /// refusals use.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Truncated => "candidate_truncated",
            Self::Empty => "candidate_empty",
            Self::NotOneFile => "candidate_not_one_file",
            Self::Tools => "candidate_tools",
            Self::Prompt(_) => "candidate_prompt",
        }
    }
}

/// The digests the reviewed closure's workload record carries for the class's three candidate
/// inputs (`files_sha256`: `TASK.md`, `base/Cargo.toml`, `base/src/lib.rs`) — the ONE pin the
/// prompt's inputs are read against (R18 A6); the profile names files, never digests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilePins {
    pub task: String,
    pub cargo: String,
    pub base: String,
}

/// The class's candidate inputs, each read once and refused unless its bytes hash to the closure's
/// pin and decode as UTF-8; the 64 KiB bound is the class directory's door (`read_declared`), the
/// prompt bound is the contract's ([`MAX_PROMPT_BYTES`]) — neither is spelled again here (A8).
#[derive(Clone, Debug)]
pub struct ClassPrompt {
    task: String,
    cargo: String,
    base: String,
}

impl ClassPrompt {
    /// # Errors
    /// `Prompt("task" | "cargo" | "base")` when that input is empty, not UTF-8 or not the pinned bytes.
    pub fn new(task: &[u8], cargo: &[u8], base: &[u8], pins: &FilePins) -> Result<Self, Refusal> {
        let read = |bytes: &[u8], pin: &str, site: &'static str| -> Result<String, Refusal> {
            if bytes.is_empty() || digest(bytes) != pin {
                return Err(Refusal::Prompt(site));
            }
            std::str::from_utf8(bytes)
                .map(str::to_owned)
                .map_err(|_| Refusal::Prompt(site))
        };
        Ok(Self {
            task: read(task, &pins.task, "task")?,
            cargo: read(cargo, &pins.cargo, "cargo")?,
            base: read(base, &pins.base, "base")?,
        })
    }
}

/// The prompt, rendered once from the class's inputs and the previous verification (F6), pure:
/// the task text; the frame's statement that it supersedes the task's return clause; the current
/// `Cargo.toml` and `src/lib.rs`; the history — `First attempt.` or the previous verdict and
/// criteria, plus the refusal's name when the previous check was the runtime's own refused-candidate
/// record — and the reply instruction.
///
/// # Errors
/// `Prompt("previous")` when a refused-candidate record cannot be read; `Prompt("render")` when the
/// rendering is past the contract's prompt bound.
pub fn render(prompt: &ClassPrompt, previous: Option<&Previous>) -> Result<String, Refusal> {
    let history = match previous {
        None => "First attempt.".to_owned(),
        Some(previous) => {
            let mut line = format!(
                "Previous attempt: {}, {} criteria satisfied",
                verdict_name(previous.verdict),
                previous.criteria
            );
            if previous.schema_id == REFUSED_CANDIDATE_SCHEMA {
                let record: serde_json::Value = serde_json::from_slice(&previous.evidence)
                    .map_err(|_| Refusal::Prompt("previous"))?;
                let refusal = record["refusal"]
                    .as_str()
                    .ok_or(Refusal::Prompt("previous"))?;
                line.push_str(", refused as ");
                line.push_str(refusal);
            }
            line.push('.');
            line
        }
    };
    let rendered = format!(
        "{task}\n\nThis request supersedes the task's return clause: do not return a patch or an \
         explanation.\n\nCurrent Cargo.toml:\n{cargo}\n\nCurrent {U64_EDITABLE}:\n{base}\n\n{history}\n\n\
         Reply with the complete contents of {U64_EDITABLE} and nothing else: no prose, no fences.\n",
        task = prompt.task,
        cargo = prompt.cargo,
        base = prompt.base,
    );
    if rendered.len() > MAX_PROMPT_BYTES {
        return Err(Refusal::Prompt("render"));
    }
    Ok(rendered)
}

const fn verdict_name(verdict: VerificationVerdict) -> &'static str {
    match verdict {
        VerificationVerdict::Passed => "passed",
        VerificationVerdict::Failed => "failed",
        VerificationVerdict::Invalid => "invalid",
        VerificationVerdict::Error => "error",
        VerificationVerdict::Timeout => "timeout",
        VerificationVerdict::Cancelled => "cancelled",
    }
}

/// The full-file grammar (G2, R18 A7): the model's text is one file — the whole text when no line
/// starts with three backticks at column 0, or, with exactly two such fence lines, the bytes from the end of the
/// opening fence's line to the start of the closing fence's line, verbatim. A truncated finish, a
/// blank text or body, and one or three-plus fence lines are refused by name. No trimming, no
/// reflow, no size rule: `apply_candidate` owns the bytes and the lines.
///
/// # Errors
/// Each [`Refusal`], named.
pub fn grammar(text: &str, finish: Finish) -> Result<Vec<u8>, Refusal> {
    if finish == Finish::Length {
        return Err(Refusal::Truncated);
    }
    // Each line's byte range; a fence is a line starting with three backticks at column 0. A blank
    // text is a blank body: the one emptiness rule sits after the fences are read.
    let mut fences: Vec<(usize, usize)> = Vec::new();
    let mut start = 0;
    while start < text.len() {
        let end = text[start..]
            .find('\n')
            .map_or(text.len(), |offset| start + offset);
        if text[start..end].starts_with("```") {
            fences.push((start, end));
        }
        start = end + 1;
    }
    let body = match fences.as_slice() {
        [] => text,
        [(_, open_end), (close_start, _)] => {
            let from = (open_end + 1).min(*close_start);
            &text[from..*close_start]
        }
        _ => return Err(Refusal::NotOneFile),
    };
    if body.trim().is_empty() {
        return Err(Refusal::Empty);
    }
    Ok(body.as_bytes().to_vec())
}

/// What one call came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// A replacement of this many bytes.
    Replacement(usize),
    Refused(Refusal),
    Provider(native::Error),
}

/// What one call to the model came to (R18 decision 6, Q2): tokens and identity as evidence, the
/// wall in the settle — kept on the source for every call until B14a-5 gives them a ledger home.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settle {
    pub attempt: String,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub wall_ms: u64,
    pub finish: Option<Finish>,
    pub identity_sha256: Option<String>,
    pub raw_sha256: Option<String>,
    pub outcome: Outcome,
}

/// The native candidate source over one pinned model install and one adapter row. It holds nothing
/// it could re-acquire from the runtime (A1): the attempt's identity, the digests, the origin, the
/// window and the cancel flag arrive with every [`Ask`].
pub struct NativeCandidates {
    profile: native::Profile,
    adapter: &'static AdapterProfile,
    prompt: ClassPrompt,
    settles: Vec<Settle>,
    /// Children an exchange left pending, retained so their custody is never dropped (A4); B14b
    /// settles them.
    retained: Vec<PendingChild>,
}

impl NativeCandidates {
    #[must_use]
    pub fn new(
        profile: native::Profile,
        adapter: &'static AdapterProfile,
        prompt: ClassPrompt,
    ) -> Self {
        Self {
            profile,
            adapter,
            prompt,
            settles: Vec::new(),
            retained: Vec::new(),
        }
    }

    /// Every call so far, in order.
    #[must_use]
    pub fn settles(&self) -> &[Settle] {
        &self.settles
    }

    /// How many pending children the source holds.
    #[must_use]
    pub fn retained(&self) -> usize {
        self.retained.len()
    }

    fn request<'a>(&self, ask: &Ask<'a>, prompt: String) -> Request<'a> {
        Request {
            invocation: Invocation {
                binding: crate::worker::Binding {
                    task: ask.task,
                    attempt: ask.attempt,
                    generation: ask.generation,
                },
                id: ask.invocation,
            },
            recipe: ask.recipe,
            workspace: ask.workspace,
            adapter_profile: self.adapter.id.to_owned(),
            selection: Selection {
                provider: native::PROVIDER.to_owned(),
                model: self.profile.model.clone(),
                effort: None,
            },
            required: Capabilities::new(&[Feature::FinalOutput, Feature::Identity, Feature::Usage]),
            prompt,
        }
    }

    /// Take every pending child out of the run's exchanges (A4) and say whether every exchange's
    /// custody settled: each report reaped its leader and its group, and left no child.
    fn custody(&mut self, run: &mut native::Run<'_>) -> bool {
        let mut settled = true;
        for exchange in &mut run.exchanges {
            if let Ok(report) = &mut exchange.result {
                if let Some(child) = report.pending.take() {
                    self.retained.push(child);
                    settled = false;
                }
                if !report.leader_reaped || !report.process_group_settled {
                    settled = false;
                }
            }
        }
        settled
    }

    fn ask(&mut self, ask: &Ask<'_>) -> Candidate {
        let attempt = ask.attempt.as_str().to_owned();
        let begun = Instant::now();
        let mut settle = Settle {
            attempt,
            input_tokens: None,
            output_tokens: None,
            wall_ms: 0,
            finish: None,
            identity_sha256: None,
            raw_sha256: None,
            outcome: Outcome::Provider(native::Error::Response),
        };
        let candidate = match render(&self.prompt, ask.previous) {
            Err(refusal) => {
                settle.outcome = Outcome::Refused(refusal);
                Candidate::Refused {
                    refusal,
                    text: Vec::new(),
                }
            }
            Ok(prompt) => {
                let request = self.request(ask, prompt);
                match native::execute(
                    &request,
                    &self.profile,
                    ask.origin,
                    ask.work_until,
                    ask.cancelled,
                ) {
                    Err(error) => {
                        settle.outcome = Outcome::Provider(error);
                        Candidate::Provider {
                            error,
                            state: ProviderState::NotDispatched,
                            cleanup_settled: true,
                            retained: self.retained.len(),
                        }
                    }
                    Ok(mut run) => {
                        let cleanup_settled = self.custody(&mut run);
                        self.judge(&run, cleanup_settled, &mut settle)
                    }
                }
            }
        };
        settle.wall_ms = u64::try_from(begun.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.settles.push(settle);
        candidate
    }

    fn judge(
        &self,
        run: &native::Run<'_>,
        cleanup_settled: bool,
        settle: &mut Settle,
    ) -> Candidate {
        let provider = |error: native::Error, settle: &mut Settle| {
            settle.outcome = Outcome::Provider(error);
            Candidate::Provider {
                error,
                state: run.provider,
                cleanup_settled,
                retained: self.retained.len(),
            }
        };
        if let Some(error) = run.error {
            return provider(error, settle);
        }
        let Some(candidate) = run.contract.candidate() else {
            return provider(native::Error::Response, settle);
        };
        settle.finish = Some(candidate.finish);
        settle.raw_sha256 = Some(digest(&candidate.raw));
        settle.identity_sha256 = candidate
            .identity
            .as_ref()
            .map(|identity| digest(&identity.raw));
        if let Usage::Reported { input, output, .. } = &candidate.usage {
            settle.input_tokens = *input;
            settle.output_tokens = *output;
        }
        let judged = if candidate.has_tool_proposals {
            Err(Refusal::Tools)
        } else {
            grammar(&candidate.text, candidate.finish)
        };
        match judged {
            Ok(bytes) => {
                settle.outcome = Outcome::Replacement(bytes.len());
                Candidate::Replacement(bytes)
            }
            Err(refusal) => {
                settle.outcome = Outcome::Refused(refusal);
                Candidate::Refused {
                    refusal,
                    text: candidate.text.clone().into_bytes(),
                }
            }
        }
    }
}

impl CandidateSource for NativeCandidates {
    fn next(&mut self, ask: &Ask<'_>) -> Candidate {
        self.ask(ask)
    }
}

#[cfg(test)]
mod tests {
    use super::{ClassPrompt, FilePins, Refusal, grammar, render};
    use crate::app::evidence::digest;
    use crate::app::runtime::{Previous, REFUSED_CANDIDATE_SCHEMA};
    use crate::store::VerificationVerdict;
    use crate::worker::Finish;
    use crate::worker::native::{ADAPTERS, FULL_FILE, PROFILE};

    fn pins(task: &[u8], cargo: &[u8], base: &[u8]) -> FilePins {
        FilePins {
            task: digest(task),
            cargo: digest(cargo),
            base: digest(base),
        }
    }

    /// R18 (a) · the rendering, whole, over two class prompts differing in every field and three
    /// histories (none; a failed check; the runtime's own refused-candidate record, whose refusal
    /// name travels); every construction refusal by name — an input that is not the closure's pinned
    /// bytes is refused at the site, and a malformed refused-candidate record is refused at render.
    #[test]
    fn the_prompt_renders_whole_from_the_pinned_inputs_and_the_previous_verdict()
    -> Result<(), Box<dyn std::error::Error>> {
        let (t1, c1, b1) = (
            &b"Change only src/lib.rs.\n"[..],
            &b"[package]\nname = \"one\"\n"[..],
            &b"pub fn parse() {}\n"[..],
        );
        let first =
            ClassPrompt::new(t1, c1, b1, &pins(t1, c1, b1)).map_err(|e| format!("{e:?}"))?;
        assert_eq!(
            render(&first, None).map_err(|e| format!("{e:?}"))?,
            "Change only src/lib.rs.\n\n\nThis request supersedes the task's return clause: do not return a patch or an explanation.\n\nCurrent Cargo.toml:\n[package]\nname = \"one\"\n\n\nCurrent src/lib.rs:\npub fn parse() {}\n\n\nFirst attempt.\n\nReply with the complete contents of src/lib.rs and nothing else: no prose, no fences.\n"
        );
        let (t2, c2, b2) = (
            &b"Reject leading zeros."[..],
            &b"[package]\nname = \"two\""[..],
            &b"// base two"[..],
        );
        let second =
            ClassPrompt::new(t2, c2, b2, &pins(t2, c2, b2)).map_err(|e| format!("{e:?}"))?;
        let failed = Previous {
            verdict: VerificationVerdict::Failed,
            criteria: 0,
            evidence: vec![1, 2, 3],
            schema_id: "hee3.receipt/1:ReceiptV1".to_owned(),
        };
        assert_eq!(
            render(&second, Some(&failed)).map_err(|e| format!("{e:?}"))?,
            "Reject leading zeros.\n\nThis request supersedes the task's return clause: do not return a patch or an explanation.\n\nCurrent Cargo.toml:\n[package]\nname = \"two\"\n\nCurrent src/lib.rs:\n// base two\n\nPrevious attempt: failed, 0 criteria satisfied.\n\nReply with the complete contents of src/lib.rs and nothing else: no prose, no fences.\n"
        );
        let refused = Previous {
            verdict: VerificationVerdict::Failed,
            criteria: 0,
            evidence: br#"{"kind":"refused_candidate","refusal":"candidate_truncated","candidate_sha256":"sha256:00"}"#.to_vec(),
            schema_id: REFUSED_CANDIDATE_SCHEMA.to_owned(),
        };
        assert!(
            render(&second, Some(&refused))
                .map_err(|e| format!("{e:?}"))?
                .contains(
                    "\n\nPrevious attempt: failed, 0 criteria satisfied, refused as candidate_truncated.\n\n"
                )
        );
        let malformed = Previous {
            evidence: b"not json".to_vec(),
            ..refused.clone()
        };
        assert_eq!(
            render(&second, Some(&malformed)).err(),
            Some(Refusal::Prompt("previous"))
        );
        let passed = Previous {
            verdict: VerificationVerdict::Passed,
            criteria: 1,
            evidence: Vec::new(),
            schema_id: "hee3.receipt/1:ReceiptV1".to_owned(),
        };
        assert!(
            render(&second, Some(&passed))
                .map_err(|e| format!("{e:?}"))?
                .contains("Previous attempt: passed, 1 criteria satisfied.")
        );
        // Construction: each input against its pin, by site.
        let good = pins(t1, c1, b1);
        assert_eq!(
            ClassPrompt::new(b"", c1, b1, &good).err(),
            Some(Refusal::Prompt("task"))
        );
        assert_eq!(
            ClassPrompt::new(t1, c1, b1, &pins(b"other", c1, b1)).err(),
            Some(Refusal::Prompt("task")),
            "bytes that are not the closure's pinned task text"
        );
        assert_eq!(
            ClassPrompt::new(t1, b"", b1, &good).err(),
            Some(Refusal::Prompt("cargo"))
        );
        assert_eq!(
            ClassPrompt::new(t1, c1, b"x", &good).err(),
            Some(Refusal::Prompt("base"))
        );
        let bad_utf8 = &[0xff_u8, 0xfe][..];
        assert_eq!(
            ClassPrompt::new(bad_utf8, c1, b1, &pins(bad_utf8, c1, b1)).err(),
            Some(Refusal::Prompt("task")),
            "pinned but not UTF-8"
        );
        // A rendering past the contract's bound is never sent.
        let big = vec![b'b'; 262_000];
        let large =
            ClassPrompt::new(t1, c1, &big, &pins(t1, c1, &big)).map_err(|e| format!("{e:?}"))?;
        assert_eq!(render(&large, None).err(), Some(Refusal::Prompt("render")));
        Ok(())
    }

    /// R18 (a) · the full-file grammar: the whole text when unfenced (a Rust file whose doc comment
    /// holds an indented three backticks is unfenced); one fenced block's body verbatim, with or without a
    /// language tag, with or without a trailing newline before the closing fence; a truncated
    /// finish, blank text, a blank body, one fence line and three fence lines refused by name — a
    /// file carrying a column-0 three backticks inside a fenced answer is three fences (the grammar's stated
    /// limit: such a file can only be sent bare).
    #[test]
    fn the_grammar_admits_exactly_one_file() {
        assert_eq!(
            grammar("pub fn a() {}\n", Finish::Stop),
            Ok(b"pub fn a() {}\n".to_vec())
        );
        let documented = "/// Example:\n///\n/// ```\n/// a();\n/// ```\npub fn a() {}\n";
        assert_eq!(
            grammar(documented, Finish::Stop),
            Ok(documented.as_bytes().to_vec()),
            "a doc-comment fence is not at column 0: the whole text is the file"
        );
        assert_eq!(
            grammar(
                "Here is the file:\n```rust\npub fn a() {}\n\nfn b() {}\n```\nDone.\n",
                Finish::Stop
            ),
            Ok(b"pub fn a() {}\n\nfn b() {}\n".to_vec())
        );
        let indented = "fn a() {}\n    ```\n    not a fence\n    ```\n";
        assert_eq!(
            grammar(indented, Finish::Stop),
            Ok(indented.as_bytes().to_vec()),
            "an indented fence line is text: the whole text is the file"
        );
        assert_eq!(
            grammar("```  \nx\n```", Finish::Stop),
            Ok(b"x\n".to_vec()),
            "a bare fence with trailing spaces; the closing fence is the last line"
        );
        assert_eq!(
            grammar("```\nno trailing newline```\n", Finish::Stop),
            Err(Refusal::NotOneFile),
            "a closing fence not at column 0 is text: one fence line"
        );
        assert_eq!(
            grammar("pub fn a() {}\n", Finish::Length),
            Err(Refusal::Truncated)
        );
        assert_eq!(grammar("", Finish::Stop), Err(Refusal::Empty));
        assert_eq!(grammar(" \n\t\n", Finish::Stop), Err(Refusal::Empty));
        assert_eq!(
            grammar("```rust\n\n```\n", Finish::Stop),
            Err(Refusal::Empty)
        );
        assert_eq!(grammar("```rust\n```\n", Finish::Stop), Err(Refusal::Empty));
        assert_eq!(
            grammar("```rust\na\n```\ntext\n```rust\nb\n```\n", Finish::Stop),
            Err(Refusal::NotOneFile)
        );
        assert_eq!(
            grammar(
                "```rust\n/// ```\n/// x\n/// ```\nfn a() {}\n```\n",
                Finish::Stop
            ),
            Ok(b"/// ```\n/// x\n/// ```\nfn a() {}\n".to_vec()),
            "doc fences inside a fenced answer start with /// not ```"
        );
        assert_eq!(
            grammar("```rust\nfn a() {}\n```\n```\n", Finish::Stop),
            Err(Refusal::NotOneFile),
            "three fence lines"
        );
        assert_eq!(
            grammar("```rust\na\n", Finish::Stop),
            Err(Refusal::NotOneFile),
            "one fence line"
        );
        let large = "x".repeat(70_000);
        assert_eq!(
            grammar(&large, Finish::Stop).map(|b| b.len()),
            Ok(70_000),
            "no size rule here: apply_candidate owns the bound"
        );
    }

    /// R18 A11 · the adapter table pinned whole: two rows, the qualified raw 512/64 profile and its
    /// templated full-file revision — the id every fixture names is the first row's.
    #[test]
    fn the_adapter_table_is_pinned_whole() {
        assert_eq!(
            ADAPTERS
                .iter()
                .map(|row| (row.id, row.num_ctx, row.num_predict, row.templated))
                .collect::<Vec<_>>(),
            vec![
                ("ollama-fc44-12ff8654/1", 512, 64, false),
                ("ollama-fc44-12ff8654/2", 4096, 1024, true),
            ]
        );
        assert_eq!(PROFILE, "ollama-fc44-12ff8654/1");
        assert_eq!(FULL_FILE.id, "ollama-fc44-12ff8654/2");
        assert_eq!(Refusal::Prompt("x").name(), "candidate_prompt");
    }
}
