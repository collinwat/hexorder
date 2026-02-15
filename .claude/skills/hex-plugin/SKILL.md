---
name: hex-plugin
description:
    Create or update plugin documentation (specs and logs). Use when starting a new plugin, updating
    a spec during implementation, logging decisions or test results, or scaffolding plugin docs for
    a shaped pitch. Also use when the user invokes /hex-plugin.
---

# Plugin

For the full lifecycle, rationale, and templates, see `docs/guides/plugin.md`.

## Which Workflow?

1. Check `docs/plugins/` for an existing plugin directory matching your work
2. If it exists → **update** the spec and log as you work
3. If it does not → **create** new plugin docs (below)

## Creating Plugin Docs

1. Create `docs/plugins/<name>/spec.md` using the template from `docs/guides/plugin.md`
2. Create `docs/plugins/<name>/log.md` using the template from `docs/guides/plugin.md`
3. Register the plugin in `docs/coordination.md`
4. If the plugin introduces shared types, use the contract skill

## Updating During Implementation

1. Add scope items to the spec as they are discovered
2. Check off success criteria as they pass
3. Record decisions in the log with context, rationale, and rejected alternatives
4. Record test results with timestamps
5. Move out-of-scope items to Deferred Items with a GitHub Issue number
