-- HEE3-ANCHORS-BEGIN
-- Anchor path: /var/home/herdr-engineering-engine-v3/migrations/004.sql
-- Navigation block. This file is an authored implementation path (original task T04): the corpus
-- publisher inventories its exact bytes and never rewrites them. Only `--` comments or blank lines
-- belong here; nothing in this block executes, and the store's identity digest covers only the SQL
-- below the end marker.
-- HEE3-ANCHORS-END

-- Migration 4 (B14-P2c; RC06/T04 additive chain): a task records the workspace it was admitted
-- for, the id of a workspace the installed class profile declares (docs/contract-decisions.md,
-- `workspace_id`), so the dispatcher reads what admission bound and never re-parses the request.
--
-- The column is added, not rebuilt: every existing row keeps every value, and reads NULL here — a
-- task admitted before this migration named no bound workspace, and the dispatcher stops it
-- `no_workspace` (B14b). The one writer (`Store::submit`) takes a `UuidV4`, so a new row cannot be
-- NULL; this CHECK refuses any other length. `store::schema` compares the table's rows over its
-- pre-step columns before and after, and refuses the step if they differ.
ALTER TABLE tasks ADD COLUMN workspace_id TEXT CHECK(workspace_id IS NULL OR length(workspace_id) = 36);
