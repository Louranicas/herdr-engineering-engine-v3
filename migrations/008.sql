-- HEE3-ANCHORS-BEGIN
-- Anchor path: /var/home/herdr-engineering-engine-v3/migrations/008.sql
-- Navigation block. This file is an authored implementation path (original task T04): the corpus
-- publisher inventories its exact bytes and never rewrites them. Only `--` comments or blank lines
-- belong here; nothing in this block executes, and the store's identity digest covers only the SQL
-- below the end marker.
-- HEE3-ANCHORS-END

-- Migration 8 (B14b-2, design R21 N13/N17 and D6 in `~/hee3-evidence/T28/B14-store-runtime-20260926/
-- DESIGN.md`; RC06/T04 additive chain). One root per bound attempt: the directory the runtime
-- materialises the attempt's workspace (`<root>/<attempt>`) and check job root
-- (`<root>/<attempt>.check`) under, recorded in the transaction that begins the attempt, so a
-- restart reads the leaves from the ledger instead of re-deriving them from its own configuration.
--
-- The key references `attempt_bindings`, not `attempts`: only a bound begin writes a root, and a root
-- without a binding is refused by the key rather than by a validate clause. The root is absolute and
-- 2 to 4096 bytes (bytes, not characters). A new table: nothing existing is rebuilt, so there is
-- nothing to preserve, and an attempt begun before this migration has no row.
CREATE TABLE attempt_paths (
    attempt_id TEXT PRIMARY KEY REFERENCES attempt_bindings(attempt_id),
    root TEXT NOT NULL CHECK(length(CAST(root AS BLOB)) BETWEEN 2 AND 4096 AND substr(root,1,1)='/')
) STRICT;
