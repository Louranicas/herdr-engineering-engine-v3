-- HEE3-ANCHORS-BEGIN
-- Anchor path: /var/home/herdr-engineering-engine-v3/migrations/002.sql
-- Navigation block. This file is an authored implementation path (original task T04): the corpus
-- publisher inventories its exact bytes and never rewrites them. Only `--` comments or blank lines
-- belong here; nothing in this block executes, and the store's identity digest covers only the SQL
-- below the end marker.
-- HEE3-ANCHORS-END

-- Migration 2 (A25; RC06/T04): `operations` admits `task.cancel` idempotency records, so the one
-- durable binding of (principal, action, version, key) to a request digest and its stored result
-- (RC03 section 6) covers cancellation too. SQLite cannot alter a CHECK, so the table is rebuilt:
-- copied under the new CHECK and renamed, inside this migration's own transaction. Nothing
-- references `operations`. Every row is preserved; `store::schema` compares the table's row count
-- and row digest before and after, and refuses the step if they differ.
CREATE TABLE operations_v2 (
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
    CHECK ((action IN ('task.submit','task.cancel') AND resource_id IS NOT NULL AND request_object IS NULL AND request_row IS NULL)
        OR (action IN ('roster.update','roster.disable') AND roster_id IS NOT NULL AND request_object IS NOT NULL)),
    PRIMARY KEY(principal_uid,principal_role,action,version,request_key)
) STRICT;
INSERT INTO operations_v2 (principal_uid,principal_role,action,version,request_key,request_digest,resource_id,roster_id,request_object,request_row,result)
    SELECT principal_uid,principal_role,action,version,request_key,request_digest,resource_id,roster_id,request_object,request_row,result FROM operations;
DROP TABLE operations;
ALTER TABLE operations_v2 RENAME TO operations;
