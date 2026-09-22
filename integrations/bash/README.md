# Bash integration

Thin command entry points over actions the engine already admits. A wrapper, and nothing
more: it builds a request, hands it to the producer, and reports the producer's verdict
unchanged. It schedules nothing, duplicates no policy, retries nothing on its own authority,
and grants no action — an action the wrapper can name is one the engine already admits, and
one it cannot name simply does not run.

```
hee3 <action> [name=value ...]     invoke one admitted action
hee3 --actions                     list the actions this wrapper can name
hee3 --inspect <action>            that action's prerequisites and the bounds in force
hee3 --version                     wrapper version and producer diagnostics
```

`HEE3_PRODUCER` names the producer executable. `HEE3_CATALOGUE` overrides where the action
vocabulary is read from; the default is `schemas/actions/control-v1.schema.json`, so an
action added to the engine is nameable here without anyone editing a list.

## Three rules, and why each is a mechanism

**No eval, and no re-parsing of untrusted strings.** Values travel in arrays from `argv` to
the producer, and the request is encoded by a JSON writer rather than built by
concatenation. A value containing a space, a quote, a backslash, a newline, `$(...)`,
backticks, `;` or `*` is a value. Fourteen cases assert on what *arrived* at the producer,
not merely that nothing exploded — a wrapper that dropped, split or executed a value would
pass a smoke test and fail every one of them.

**A missing producer is a failure.** Unset, absent or non-executable each exit 3 with a
diagnostic. Printing nothing and exiting 0 because the wrapped thing was absent is the worst
outcome available, so it is not reachable.

**The producer's exit status survives every pipe.** This is the rule the file exists for, and
it is where the first version was wrong.

## The bug the tests found

A pipeline reports its **last** element's status. `producer | head -c N` is therefore green
whenever `head` succeeds — which is always — so a producer that printed a partial result and
exited 7 looked like a success.

The first version of `invoke` got this wrong twice over:

```bash
output=$(printf '%s' "$request" | "$bin" "$action" | head -c "$MAX") || true
local -a stages=("${PIPESTATUS[@]}")
```

`$(...)` is a subshell, so the parent's `PIPESTATUS` describes the *assignment*, not the
pipeline. And `|| true` makes `true` the last command, which resets `PIPESTATUS` to `(0)`.
Ten of the fifty-two cases went red, including `a failing producer fails the wrapper through
the pipe` — the one written for exactly this.

The pipeline now runs in this shell, with its output going to a file, and `stages[1]` is read
before anything else touches `PIPESTATUS`. Every producer code from 1 to 127 is passed
through unchanged, a producer killed by a signal is not a success, and a producer that
overruns the output bound dies of `SIGPIPE` rather than being quietly truncated into one.

A second, smaller edit came from the same direction: `output=$(cat file)` strips trailing
newlines, so a producer emitting `{}\n\n\n` had one silently removed. The wrapper now `cat`s
the file and adds a final newline only when the producer did not end with one. A wrapper that
edits its producer's bytes is not reporting its producer's result.

## Bounds

| Bound | Value | On overrun |
|---|---|---|
| arguments | 64 | exit 4 |
| bytes per `name=value` | 8192 | exit 4 |
| producer stdout | 1 MiB | bounded in the stream; the producer's own status is reported |

Each bound is checked from both sides: 64 arguments must be accepted and 65 refused, or only
the refusing half was ever tested.

Exit codes: `0` producer succeeded · `2` usage · `3` missing producer · `4` bounds ·
otherwise the producer's own code, unchanged.

## Scope

`shellcheck` is not installed in this habitat, so the syntax check here is `bash -n`, which
cannot see a missing `]`. That gap is stated rather than papered over; the 53 cases in
`tests/bash_wrapper.py` drive the real script end to end against real producer fixtures,
which is the stronger evidence available today.

The suite invokes the wrapper as `bash integrations/bash/hee3`, not by executing it. The
corpus publisher owns this file's mode — it is listed in `corpus/publication-outputs.json`
and normalised to 0644 on every generation — so depending on the executable bit made 54 cases
fail with `PermissionError` after a routine publish. `tools/check-quality` is invoked through
`python3` for the same reason. The shebang is still pinned by a case, because it is what makes
the file runnable for anyone who does install it with the bit.

No transport, scheduler, grant, acceptance or admission is implemented here.
