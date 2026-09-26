-- HEE3-ANCHORS-BEGIN
-- Anchor path: /var/home/herdr-engineering-engine-v3/migrations/007.sql
-- Navigation block. This file is an authored implementation path (original task T04): the corpus
-- publisher inventories its exact bytes and never rewrites them. Only `--` comments or blank lines
-- belong here; nothing in this block executes, and the store's identity digest covers only the SQL
-- below the end marker.
-- HEE3-ANCHORS-END

-- Migration 7 (B14a-5, design R19 in `~/hee3-evidence/T28/B14-store-runtime-20260926/DESIGN.md`;
-- RC06/T04 additive chain). One change: the worker's settle joins the run records an attempt's
-- settle commits, as the kind 'worker_settle' (schema hee3.worker-settle/1, compiled beside the
-- store's RunRecordKind so a kind/schema mismatch stays unrepresentable). SQLite cannot alter a
-- CHECK (DS1 C3), so attempt_records is copied under the widened CHECK and renamed inside this
-- migration's own transaction, as 002 and 003 rebuilt operations; every row is preserved and
-- `store::schema` compares the table's rows before and after (`Preserved { columns: "*" }`). The
-- index is recreated under its name. No table references attempt_records.
CREATE TABLE attempt_records_v7 (
    event_id TEXT NOT NULL REFERENCES events(id),
    kind TEXT NOT NULL CHECK(kind IN ('run_clock','run_outcome','run_cleanup','readbacks','capture','worker_settle')),
    attempt_id TEXT NOT NULL REFERENCES attempts(id),
    digest TEXT NOT NULL REFERENCES artifacts(digest),
    artifact_id TEXT NOT NULL CHECK(length(CAST(artifact_id AS BLOB)) = 36),
    PRIMARY KEY(event_id, kind)
) STRICT, WITHOUT ROWID;
INSERT INTO attempt_records_v7 (event_id,kind,attempt_id,digest,artifact_id)
    SELECT event_id,kind,attempt_id,digest,artifact_id FROM attempt_records;
DROP TABLE attempt_records;
ALTER TABLE attempt_records_v7 RENAME TO attempt_records;
CREATE INDEX attempt_records_by_attempt ON attempt_records(attempt_id);
