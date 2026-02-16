# Plugin Log: Shortcuts

## Status: speccing

## Decision Log

### 2026-02-16 — New plugin + contract (not extend editor_ui)

**Context**: Shortcut registry is consumed by multiple plugins (camera, persistence, hex_grid,
editor_ui). **Decision**: Create a new `shortcuts` plugin with its own `shortcuts` contract.
**Rationale**: Per the constitution, shared types must live in contracts. The registry logic
warrants its own module. **Alternatives rejected**: Extending editor_ui (would bloat it, wrong
separation of concerns).

### 2026-02-16 — TOML config format

**Context**: Pitch said "JSON or TOML." **Decision**: TOML. **Rationale**: Consistent with Rust
ecosystem (Cargo.toml), already linted by taplo, human-editable.

### 2026-02-16 — Compile-time config path

**Context**: Where to store shortcuts.toml? **Decision**: Compile-time cfg flag — local dev uses
`./config/shortcuts.toml`, macOS app uses `~/Library/Application Support/hexorder/shortcuts.toml`.
**Rationale**: No `dirs` crate dependency needed; paths are simple constants.

### 2026-02-16 — Single CommandExecutedEvent with CommandId

**Context**: How to execute commands across plugins. **Decision**: Single `CommandExecutedEvent`
with string-based `CommandId`. Each plugin observes and matches. **Rationale**: Scales better than
separate event types per command. String matching cost is negligible.

### 2026-02-16 — Fuzzy matching crate from the start

**Context**: Command palette search behavior. **Decision**: Add fuzzy matching crate (sublime_fuzzy
or nucleo) immediately rather than starting with substring. **Rationale**: Users expect fuzzy
matching from a command palette. Crate choice confirmed during research spike.

### 2026-02-16 — Target ~25-30 commands initially

**Context**: How many commands to register beyond the 14 migrated shortcuts. **Decision**: ~25-30
total including tool switching, view toggles, mode switching, edit actions. **Rationale**: Palette
should feel comprehensive and discoverable on day one.

### 2026-02-16 — Research spike before custom implementation

**Context**: Issue #25 (shortcut management libraries) is still open. **Decision**: Quick spike to
evaluate leafwing-input-manager and alternatives before building custom. **Rationale**: Ensure we
are not reinventing the wheel. If a crate fits, adopt it.

## Test Results

_No test runs yet._

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
|         |            |        |          |

## Deferred / Future Work

- [None yet]

## Status Updates

| Date       | Status   | Notes                                     |
| ---------- | -------- | ----------------------------------------- |
| 2026-02-16 | speccing | Initial spec created, design doc reviewed |
