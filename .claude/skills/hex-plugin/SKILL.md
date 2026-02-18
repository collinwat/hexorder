---
name: hex-plugin
description:
    Create or update plugin documentation (specs and logs). Use when starting a new plugin, updating
    a spec during implementation, logging decisions or test results, or scaffolding plugin docs for
    a shaped pitch. Also use when the user invokes /hex-plugin.
---

# Plugin

Create or update plugin documentation — specs and logs — throughout the plugin lifecycle.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name           | Value                                      | Description                                           |
| -------------- | ------------------------------------------ | ----------------------------------------------------- |
| `project_root` | repository root                            | Base directory; all paths are relative to this        |
| `plugin_guide` | `{{ project_root }}/docs/guides/plugin.md` | Plugin lifecycle, spec and log templates, conventions |
| `plugins_dir`  | `{{ project_root }}/docs/plugins`          | Plugin documentation directory                        |
| `architecture` | `{{ project_root }}/docs/architecture.md`  | Plugin registration and dependency graph              |

## 1. Learn the Plugin Lifecycle

Read `{{ plugin_guide }}` to extract the spec template, log template, and lifecycle conventions.
Specifically, find and hold in memory:

- **Spec template** — the required structure for plugin spec documents
- **Log template** — the required structure for plugin log documents
- **Lifecycle phases** — how plugins move through creation, implementation, and completion
- **Naming and scope conventions** — how to name plugins and define scope boundaries

Do NOT hardcode template structure or lifecycle rules — always read them fresh from the file.

## Which Workflow?

1. Check `{{ plugins_dir }}` for an existing plugin directory matching your work
2. If it exists → **update** the spec and log as you work
3. If it does not → **create** new plugin docs (below)

## Creating Plugin Docs

1. Create `{{ plugins_dir }}/<name>/spec.md` using the spec template extracted from
   `{{ plugin_guide }}`
2. Create `{{ plugins_dir }}/<name>/log.md` using the log template extracted from
   `{{ plugin_guide }}`
3. Register the plugin in `src/main.rs` and `{{ architecture }}`
4. If the plugin introduces shared types, use the contract skill

## Updating During Implementation

1. Add scope items to the spec as they are discovered
2. Check off success criteria as they pass
3. Record decisions in the log with context, rationale, and rejected alternatives
4. Record test results with timestamps
5. Move out-of-scope items to Deferred Items with a GitHub Issue number
