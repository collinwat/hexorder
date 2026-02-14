---
name: feature
description:
    Create or update feature documentation (specs and logs). Use when starting a new feature,
    updating a spec during implementation, logging decisions or test results, or scaffolding feature
    docs for a shaped pitch. Also use when the user invokes /feature.
---

# Feature

For the full lifecycle, rationale, and templates, see `docs/guides/feature.md`.

## Which Workflow?

1. Check `docs/features/` for an existing feature directory matching your work
2. If it exists → **update** the spec and log as you work
3. If it does not → **create** new feature docs (below)

## Creating Feature Docs

1. Create `docs/features/<name>/spec.md` using the template from `docs/guides/feature.md`
2. Create `docs/features/<name>/log.md` using the template from `docs/guides/feature.md`
3. Register the feature in `docs/coordination.md`
4. If the feature introduces shared types, use the contract skill

## Updating During Implementation

1. Add scope items to the spec as they are discovered
2. Check off success criteria as they pass
3. Record decisions in the log with context, rationale, and rejected alternatives
4. Record test results with timestamps
5. Move out-of-scope items to Deferred Items with a GitHub Issue number
