-- HEE3-ANCHORS-BEGIN
-- Anchor path: /var/home/herdr-engineering-engine-v3/migrations/003.sql
-- Navigation block. This file is an authored implementation path (original task T04): the corpus
-- publisher inventories its exact bytes and never rewrites them. Only `--` comments or blank lines
-- belong here; nothing in this block executes, and the store's identity digest covers only the SQL
-- below the end marker.
-- HEE3-ANCHORS-END

-- Migration 3 (B08; RC03 section 6 `task.resolve`; RC06/T04 additive chain): an operator's
-- disposition of an unresolved obligation is recorded, never written over what was observed.
--
-- 1. `task_dispositions` holds each disposition: the obligation it names (an attempt, or a delivery
--    event), what was decided, whether it closed the obligation, the operator's reason and evidence
--    references, and the event that records it. At most one disposition closes an obligation.
-- 2. `operations` admits `task.resolve` idempotency records (rebuilt as migration 2 rebuilt it).
-- 3. `task_stops` admits `abandoned`: an operator's abandonment stops a task through the one stop
--    door, with a stop row, so it is as final as any other stop.
-- SQLite cannot alter a CHECK, so both tables are copied under the new CHECK and renamed, inside
-- this migration's own transaction. No table references either. Every row is preserved;
-- `store::schema` compares each table's rows before and after, and refuses the step if they differ.
CREATE TABLE task_dispositions (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    obligation_kind TEXT NOT NULL CHECK(obligation_kind IN ('attempt','delivery')),
    obligation_id TEXT NOT NULL,
    disposition TEXT NOT NULL CHECK(disposition IN ('retry','abandon','acknowledge_external_effect','quarantine')),
    resolves INTEGER NOT NULL CHECK(resolves IN (0,1)),
    reason TEXT NOT NULL CHECK(length(CAST(reason AS BLOB)) BETWEEN 1 AND 2048),
    evidence BLOB NOT NULL,
    principal_uid INTEGER NOT NULL CHECK(principal_uid BETWEEN 0 AND 4294967295),
    principal_role TEXT NOT NULL,
    event_id TEXT NOT NULL UNIQUE REFERENCES events(id)
) STRICT;
CREATE UNIQUE INDEX one_resolution ON task_dispositions(obligation_kind,obligation_id) WHERE resolves=1;

CREATE TABLE operations_v3 (
    principal_uid INTEGER NOT NULL,
    principal_role TEXT NOT NULL,
    action TEXT NOT NULL,
    version INTEGER NOT NULL CHECK(version=1),
    request_key TEXT NOT NULL,
    request_digest TEXT NOT NULL,
    resource_id TEXT REFERENCES tasks(id),
    roster_id TEXT REFERENCES roster_records(id),
    request_object TEXT REFERENCES artifacts(digest),
    request_row INTEGER CHECK(request_row BETWEEN 0 AND 255),
    result BLOB NOT NULL,
    CHECK ((resource_id IS NULL) <> (roster_id IS NULL)),
    CHECK ((action IN ('task.submit','task.cancel','task.resolve') AND resource_id IS NOT NULL AND request_object IS NULL AND request_row IS NULL)
        OR (action IN ('roster.update','roster.disable') AND roster_id IS NOT NULL AND request_object IS NOT NULL)),
    PRIMARY KEY(principal_uid,principal_role,action,version,request_key)
) STRICT;
INSERT INTO operations_v3 (principal_uid,principal_role,action,version,request_key,request_digest,resource_id,roster_id,request_object,request_row,result)
    SELECT principal_uid,principal_role,action,version,request_key,request_digest,resource_id,roster_id,request_object,request_row,result FROM operations;
DROP TABLE operations;
ALTER TABLE operations_v3 RENAME TO operations;

CREATE TABLE task_stops_v3 (
    task_id TEXT PRIMARY KEY REFERENCES tasks(id),
    event_id TEXT NOT NULL UNIQUE REFERENCES events(id),
    evidence_digest TEXT NOT NULL REFERENCES artifacts(digest),
    reason TEXT NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('failed','cancelled','abandoned'))
) STRICT;
INSERT INTO task_stops_v3 (task_id,event_id,evidence_digest,reason,state)
    SELECT task_id,event_id,evidence_digest,reason,state FROM task_stops;
DROP TABLE task_stops;
ALTER TABLE task_stops_v3 RENAME TO task_stops;
