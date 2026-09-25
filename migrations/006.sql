-- HEE3-ANCHORS-BEGIN
-- Anchor path: /var/home/herdr-engineering-engine-v3/migrations/006.sql
-- Navigation block. This file is an authored implementation path (original task T04): the corpus
-- publisher inventories its exact bytes and never rewrites them. Only `--` comments or blank lines
-- belong here; nothing in this block executes, and the store's identity digest covers only the SQL
-- below the end marker.
-- HEE3-ANCHORS-END

-- Migration 6 (B09b + DS2 + B17; RC06/T04 additive chain; decision record DS1-DS2.md
-- `~/hee3-evidence/T00-plan-20260926/`). Additive only: every existing table is extended with ADD
-- COLUMN and one table is created, so no row is rebuilt and `store::schema` compares each extended
-- table over its pre-step columns before and after.
--
-- 1 · Evidence identity (B09b). A committed object is named by a digest; a receipt names it by an
--     artifact id, a media type and a schema id as well. The three are stored together or not at
--     all; each is required NOT NULL in the second arm, since a CHECK passes on NULL and
--     `length(NULL)` would otherwise let a partial identity through; the digest and byte length are derived from the object row, never stored twice. A row
--     written before this migration reads NULL identity, and a view over it is refused
--     `EvidenceIdentity` (as B09a already does).
ALTER TABLE verifications ADD COLUMN evidence_artifact_id TEXT;
ALTER TABLE verifications ADD COLUMN evidence_media_type TEXT;
ALTER TABLE verifications ADD COLUMN evidence_schema_id TEXT
    CHECK((evidence_artifact_id IS NULL AND evidence_media_type IS NULL AND evidence_schema_id IS NULL)
          OR (evidence_artifact_id IS NOT NULL AND evidence_media_type IS NOT NULL
              AND evidence_schema_id IS NOT NULL
              AND length(evidence_artifact_id) = 36
              AND length(evidence_media_type) BETWEEN 1 AND 128
              AND length(evidence_schema_id) BETWEEN 1 AND 128));
ALTER TABLE task_stops ADD COLUMN evidence_artifact_id TEXT;
ALTER TABLE task_stops ADD COLUMN evidence_media_type TEXT;
ALTER TABLE task_stops ADD COLUMN evidence_schema_id TEXT
    CHECK((evidence_artifact_id IS NULL AND evidence_media_type IS NULL AND evidence_schema_id IS NULL)
          OR (evidence_artifact_id IS NOT NULL AND evidence_media_type IS NOT NULL
              AND evidence_schema_id IS NOT NULL
              AND length(evidence_artifact_id) = 36
              AND length(evidence_media_type) BETWEEN 1 AND 128
              AND length(evidence_schema_id) BETWEEN 1 AND 128));
ALTER TABLE acceptance_objects ADD COLUMN artifact_id TEXT;
ALTER TABLE acceptance_objects ADD COLUMN media_type TEXT;
ALTER TABLE acceptance_objects ADD COLUMN schema_id TEXT
    CHECK((artifact_id IS NULL AND media_type IS NULL AND schema_id IS NULL)
          OR (artifact_id IS NOT NULL AND media_type IS NOT NULL AND schema_id IS NOT NULL
              AND length(artifact_id) = 36
              AND length(media_type) BETWEEN 1 AND 128
              AND length(schema_id) BETWEEN 1 AND 128));
-- The ledger mints the manifest's media type and schema as constants, so only its id is stored.
ALTER TABLE acceptances ADD COLUMN manifest_artifact_id TEXT
    CHECK(manifest_artifact_id IS NULL OR length(manifest_artifact_id) = 36);

-- 2 · Run records (DS2): the settle that observed an attempt commits the records of that run in its
--     own transaction, keyed by its own event, so nothing earlier, later or of another attempt can
--     stand in for them. At most one record of each kind per observation (the key); the kind names
--     the schema, compiled beside the store's `RecordKind`. `settled_event` names the observation
--     that settled the attempt: the only record set a composer may read.
ALTER TABLE attempts ADD COLUMN settled_event TEXT REFERENCES events(id);
CREATE TABLE attempt_records (
    event_id TEXT NOT NULL REFERENCES events(id),
    kind TEXT NOT NULL CHECK(kind IN ('run_clock','run_outcome','run_cleanup','readbacks','capture')),
    attempt_id TEXT NOT NULL REFERENCES attempts(id),
    digest TEXT NOT NULL REFERENCES artifacts(digest),
    artifact_id TEXT NOT NULL CHECK(length(artifact_id) = 36),
    PRIMARY KEY(event_id, kind)
) STRICT, WITHOUT ROWID;
CREATE INDEX attempt_records_by_attempt ON attempt_records(attempt_id);

-- 3 · Loop progress (B17): the criteria a check satisfied, as the 64-bit pattern in 16 lower-hex
--     digits (a u64 above i64::MAX cannot be an INTEGER here). It rides in this migration because
--     B09b already changes `record_verification`: a later column would leave every verification
--     recorded in between without it, and a task in flight across that window could never resume
--     its loop.
ALTER TABLE verifications ADD COLUMN satisfied_criteria TEXT
    CHECK(satisfied_criteria IS NULL
          OR (length(satisfied_criteria) = 16 AND satisfied_criteria NOT GLOB '*[^0-9a-f]*'));

-- 4 · The disposition scan by task, scheduled "with B09b's migration" (B09 design R2.6).
CREATE INDEX task_dispositions_by_task ON task_dispositions(task_id);
