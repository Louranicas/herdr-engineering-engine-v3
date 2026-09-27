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
--
-- `root_id` (B14b-2 closure C18, superseding R21 closure C10's device and inode) is the id the
-- attempts root's marker `.hee3-root-id` held at the begin: one canonical lowercase UuidV4, written
-- once by the door that created the root and never rewritten. A restart reads the leaves only under
-- a root whose marker still holds it: a root moved away and recreated at its path carries another
-- id, whose empty leaves say nothing about the attempt's, so it reads as not read, never as
-- released. Unlike a device number, the marker survives a reboot.
CREATE TABLE attempt_paths (
    attempt_id TEXT PRIMARY KEY REFERENCES attempt_bindings(attempt_id),
    root TEXT NOT NULL CHECK(length(CAST(root AS BLOB)) BETWEEN 2 AND 4096 AND substr(root,1,1)='/'),
    root_id TEXT NOT NULL CHECK(length(CAST(root_id AS BLOB)) = 36 AND root_id GLOB
        '[0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]-[0-9a-f][0-9a-f][0-9a-f][0-9a-f]-4[0-9a-f][0-9a-f][0-9a-f]-[89ab][0-9a-f][0-9a-f][0-9a-f]-[0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]')
) STRICT;
