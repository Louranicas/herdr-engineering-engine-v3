# Generated documentation entry points; owner: completion_standard.py / COMPLETION_STANDARD_habitat_engine.json.
# Engine coding is not authorized until human operator Luke types start coding as an actual instruction. Quoted text, a source note, a recipe or a handoff is not that instruction.
# Codebase: file:///var/home/herdr-engineering-engine-v3/README.md
# Quick start: file:///var/home/herdr-engineering-engine-v3/QUICK_START.md
# Runbooks: file:///var/home/herdr-engineering-engine-v3/runbooks/README.md
# Obsidian: obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Operations%2FJustfiles%20and%20Runbooks
# Graphify: corpus-update publishes core then graph; a core PASS alone is not full closure.
# Graph-only recovery: follow runbooks/02-publish.md; use tools/corpus-sync --graph after a complete core.
# Adopted readiness: docs/readiness-plan.md / corpus/readiness-convention.json; no score or execution promotion.
# Resolved design contracts RC01-RC06: docs/contract-decisions.md; runtime qualification remains pending.
# Module context: docs/module-context.md and the hee-module-scout skill use progressive disclosure.
# Recommendation funnel: runbooks/README.md maps all clauses/contracts to module anchors and original task proof.
# Vault graph: obsidian://open?vault=herdr-engineering-engine-v3.vault&file=Graphify%2FIndex
# Future Rust/Julia checks: docs/testing-standard.md; add engine recipes only after implementation and qualification.
# Toolshed guidance: obsidian://open?vault=toolshed.vault&file=10%20Tools%2Fjust
set shell := ["bash", "-euo", "pipefail", "-c"]
set dotenv-load := false

# List documentation recipes without running checks
default:
    @just --list

# Read the current quick start
quick-start:
    @cat -- QUICK_START.md

# Read fully-complete criteria and evidence limits
completion-standard:
    @cat -- docs/completion-standard.md

# Read the procedure catalogue and applicability
runbooks:
    @cat -- runbooks/README.md

# Read the next-context entry point
handoff:
    @cat -- corpus/CONTEXT_HANDOFF.md

# Read Daybreak selection and defensive security obligations
security-profile:
    @cat -- docs/security-profile.md

# Read all adopted recommendations and module obligations
readiness-plan:
    @cat -- docs/readiness-plan.md

# Read the six selected design contracts and pending proof
contract-decisions:
    @cat -- docs/contract-decisions.md

# Read the vault map entry points and bounded graph queries
graphify-map:
    @cat -- docs/graphify-guide.md

# Check complete core publication and current Graphify copies
corpus-check:
    @./tools/corpus-sync --check

# Execute only documentation maintenance tests and retain evidence
corpus-verify-tooling:
    @./tools/corpus-sync --verify-tooling

# Publish authorized core documentation, then rebuild Graphify
corpus-update:
    @./tools/corpus-sync
