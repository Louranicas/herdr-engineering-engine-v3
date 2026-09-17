# T05 roster candidate boundary

The existing Store owns roster records, immutable revisions, operation replay,
observations, instance history, attempt pins and cancellation causes in the same
SQLite ledger as tasks. This is an in-process development candidate. T06 owns
actual execute/verify/repair integration; T28 owns external action/grant/cursor
enforcement. No module admission or provider/source attestation follows from the
presence of this API.

## Update authority and source identity

The coordinator supplies an authenticated `Principal`; mutation methods require
the operator role. UID and role define visibility and replay scope. The definition's
`owner_id` is a label and never authenticates the principal. Reads do not disclose
another principal's records, observations, instances or operation outcomes.

`roster::parse_changes` decodes a closed TOML document and returns validated
`contracts::roster::Update` values. The trusted coordinator passes those values
and **the same exact source bytes** to `Store::roster_apply` with
`RequestSource::Import`. The native decoder instead passes its validated single
row with `RequestSource::Native` and the exact admitted JSON object excluding LF.
The Store facade does not implement that future native protocol decoder.

The version-1 change manifest contains `kind = "hee3-roster-changes"`, `version = 1`
and `updates`. Each row has an idempotency key, audit reason and definition.
Create omits both `record_id` and `expected_revision`; an update supplies both.
TOML absence maps to null only for these nullable fields and `endpoint_ref`.
Definitions preserve RC03's UTF-8 byte bounds, ASCII controls and repeated
capability labels. Unknown fields, raw credentials and endpoint URLs are invalid.

Import keys bind SHA-256 of `HEE3-roster-import/1`, NUL, the four-byte big-endian
zero-based row ordinal and the entire exact UTF-8 manifest. Formatting, row order
or sibling changes under reused keys conflict. Native replay retains RC03's exact
raw-frame digest without canonicalization. The Store retains source bytes once
as an immutable object and references it from each operation with the ordinal.

Every row is validated before any mutation; all revision predicates are checked
again inside one immediate transaction. Invalid/stale batches change no admitted
record, event, revision, operation or artifact registration. Prepublished orphan
objects may remain under the existing retention policy. Replay returns the
original outcome; current inspect separately reports later changes. Updates
preserve the disabled bit, and omission never means deletion or re-enablement.

## Profiles, observations and instances

The closed RC03 definition is the reusable profile. A process instance separately
retains its task/attempt, incarnation, revision, session/workspace references,
start/lease, state and optional reported usage. PIDs and pane labels are not
identity or readiness inputs. No import, export or query launches a process,
probes an endpoint or resolves a credential.

Store stamps observation receipt time using one process-local receiving clock:
a fresh random UUID epoch, checked monotonic elapsed milliseconds and a Unix
origin. Incoming observations cannot supply receiver timestamps. Reopen creates
a new receiver epoch, retaining historical facts while invalidating their current
freshness. Sender time is evidence only; future sender time cannot qualify proof.
Age must be strictly less than a TTL in 1..60,000 ms. Missing, regressed, expired,
wrong-epoch or incorrectly bound facts remain unknown. Sequence orders receipts
that occur in the same millisecond.

The generic observation entry records an unconfirmed source claim. Designated
worker, service-probe and provider-response return paths stamp source confirmation
out of band and refuse a conflicting input source. Those methods assume a trusted
coordinator/owner caller; their presence does not qualify the future runtime
producer or authorize an external caller to choose such a path. The evidence UUID
is an opaque reference here; T06 and the integrated owners must establish actual
evidence custody and availability.

Instance-bound observations validate against that instance's admitted historical
profile revision and pin. They are retained separately and cannot replace current
profile availability. Historical observation readback reapplies principal scope.
Beginning a new rostered attempt requires each requested capability in both the
selected declaration and fresh, confirmed, available profile evidence, subject
to exact revision/version/locality constraints. It commits the existing task
attempt, immutable pins and a starting instance atomically. It does not launch
or settle a worker. Later profile changes do not rewrite those pins.

Disable serializes against this admission transaction. `leave_running` creates
no cancellation cause. `request_cancel` includes every unsettled owned attempt
and the latest settled attempt while its task is still verifying. Earlier settled
attempts and repair-pending or terminal tasks are not revived. Each cause retains
the disable event and profile while the task owner records cancellation intent
idempotently. Two disabled pinned profiles retain two causes; replay duplicates
neither. Cancellation intent does not release unknown usage, effects or cleanup.

## Bounds and retained snapshots

The first profile allows 256 roster records, 256 import rows, 1 MiB source input,
1..16 selections per new attempt, and 4,096 retained entries in each bounded
revision/observation/instance/pin/cancellation history. Capacity pressure refuses
new admission and does not evict historical proof. Instance lists refuse a result
over 256 rows rather than silently truncate it. Export is limited to 32 MiB.

Store captures visible records, a ledger cutoff and a receiving-clock sample in
one read transaction. Pure query pages borrow that value; later writes cannot
alter it. Empty capability requirements expose inventory with explicit freshness;
they do not imply eligibility. Public opaque cursor issuance remains T28.

Export uses `hee3-roster-snapshot`, stable record-ID order and revision/disabled
state. It has no operation keys and is rejected as an import. Both shipped TOML
files contain explicit empty change manifests. Restart reads SQLite and never
implicitly overlays either file.

## Schema, dependencies and verification scope

Schema 1 remains unreleased. Its exact draft checksum changes with this candidate;
old draft ledgers refuse normal opening without rebasing or rewriting history.
T04's original source/evidence remains archived. Migration freeze and supported
operational upgrades/restore remain T06/T18 obligations. Store-only backup now
includes all roster tables and registered source objects, with the same
unqualified restore status as T04.

The pinned TOML crate is 1.1.5+spec-1.1.0 with only std/serde/parse/display enabled.
Its bounded parser keeps `unbounded` disabled. The existing rustix 1.1.4 adds
`rand`; UUID allocation uses nonblocking getrandom under the operation deadline,
with no insecure fallback. SQLite remains the exact reviewed 3.53.4 static input.
All operations retain the original Store deadline, transaction failure and
uncertain-commit rules. Independent development tests and mutation evidence are
recorded separately; candidate-authored counts do not admit a module.
