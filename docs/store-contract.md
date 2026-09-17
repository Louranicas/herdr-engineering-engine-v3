# Transactional store contract

Original task: T04. One `Store` owns the selected generation's SQLite connection and stable root lock. Trusted coordinator modules supply authenticated principal identity, exact task/attempt revisions, the original deadline and verified results. The facade does not authenticate peers or qualify a verifier.

## Durable behavior

- Admission persists principal-scoped request replay, task, conservative work/verification reservations and event in one immediate transaction. Exact replay returns the original admission; changed request bytes conflict. A stable request key recovers a lost first reply.
- Task revisions and attempt generations are separate. At most three attempts are allowed; unfinished effect, usage or cleanup custody prevents reuse. Cancellation is durable intent and does not settle an external effect. Known usage, committed effects and settled cleanup cannot be erased by a later uncertain observation.
- Each transaction uses the caller's deadline, at most five seconds of SQLite busy waiting, and a progress callback for long database work. Failed or uncertain commit poisons further mutations; principal-scoped reads remain available. No whole action is retried automatically.
- Acceptance uses an event UUID allocated before immutable manifest construction. Objects and manifest are published and synced first. A short transaction rechecks current task/attempt/criteria, cancellation, proof availability and verification allowance, then commits acceptance, object references, event, budget and delivery outbox together. Historical acceptance survives later proof loss.

## Filesystem and database profile

The cooperative TH-DEV contract requires an existing canonical private root, one owner, Linux file/directory fsync and atomic same-filesystem rename with no replacement. Descriptor-relative object operations reject substituted file types, links and broad permissions. A stable lock inode is retained across restarts. Exclusive stage files are synced with final metadata, published, and followed by directory sync; failure removes only an owned unpublished stage. Published but unreferenced objects remain available for later reconciliation. Device power-loss behavior and hostile same-UID isolation require later qualification.

The development build uses the retained SQLite 3.53.4 amalgamation, linked statically through rusqlite 0.40.2 / libsqlite3-sys 0.38.2. Runtime version, source identity and required compile options are checked. WAL/FULL, foreign keys, normal locking, disabled read-uncommitted, checkpoint threshold and defensive connection settings are verified. No ambient SQLite lookup or bundled alternate SQLite runtime is used by the reviewed quality runner.

Migration 001 is currently an unreleased draft. Its exact whole-file checksum, predecessor and package identity are recorded transactionally and compared on reopen, together with schema, integrity, foreign-key and acceptance/outbox invariants. Unrelated, altered or future schemas refuse. Before migration/module release, the existing corpus owner must freeze accepted migration bytes and move mutable navigation to a sidecar; the current draft must not be represented as a released migration.

## Store snapshot boundary

`hee3-store-backup/1` means a store snapshot only. Backup requires quiescent attempt/effect/cleanup custody and a fresh private destination. SQLite Backup API steps reach Done; the wrapper calls finish through Drop, whose numeric return is not observable through rusqlite. The closed copy is reopened and checked, all registered objects are copied and rehashed, and a manifest is published last. Inspection rechecks manifest, database, logical inventory, byte bounds and every object.

The report remains `RestoreStatus::Unqualified`. T18 must add package/configuration/service identity, recovery fencing, independent restore and post-cutoff external-effect reconciliation before normal operational restoration is admitted.

## Verification ownership

`tests/t04_store.rs` contains 68 independent store-primary test functions. They use literal arithmetic/state oracles, known digest vectors, separate committed SQLite queries, real close/reopen and competing-writer behavior, and test-only failure cut points. Profile repetitions add no distinct cases. The development matrix and deliberate mutation controls retain their exact sources, commands, raw results and cleanup observations. T06 owns protected collection and actual execute/check/repair integration; source presence and a passing development suite do not admit the module.
