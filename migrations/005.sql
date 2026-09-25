-- HEE3-ANCHORS-BEGIN
-- Anchor path: /var/home/herdr-engineering-engine-v3/migrations/005.sql
-- Navigation block. This file is an authored implementation path (original task T04): the corpus
-- publisher inventories its exact bytes and never rewrites them. Only `--` comments or blank lines
-- belong here; nothing in this block executes, and the store's identity digest covers only the SQL
-- below the end marker.
-- HEE3-ANCHORS-END

-- Migration 5 (B14a-1a; RC06/T04 additive chain): an attempt begun for an installed workspace
-- records what it was bound to — the content digests of the baseline and protected snapshots it
-- captured at dispatch and of the class profile it was dispatched under — in the same transaction
-- that begins it, so the attempt cannot exist without its binding.
--
-- A task's attempts are all bound or all unbound: the bound door refuses a task with an unbound
-- attempt, the unbound doors refuse a task with a binding, and acceptance refuses an unbound attempt
-- of a bound task (`store`, one rule in the transaction that commits). A new table: nothing existing
-- is rebuilt, so there is nothing to preserve.
CREATE TABLE attempt_bindings (
    attempt_id TEXT PRIMARY KEY REFERENCES attempts(id),
    task_id TEXT NOT NULL REFERENCES tasks(id),
    baseline_digest TEXT NOT NULL CHECK(length(baseline_digest) = 71),
    protected_digest TEXT NOT NULL CHECK(length(protected_digest) = 71),
    profile_digest TEXT NOT NULL CHECK(length(profile_digest) = 71)
) STRICT;
CREATE INDEX attempt_bindings_by_task ON attempt_bindings(task_id);
