# Plugin Log: Shortcuts

## Status: building

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

### 2026-02-16 — Research spike: proceed custom (closes #25)

**Context**: Issue #25 (shortcut management libraries) is still open. Evaluated 5 crates.
**Decision**: Proceed with custom HashMap-based ShortcutRegistry. No crate adopted. **Rationale**:
Only 2 viable crates (leafwing-input-manager v0.20, bevy_enhanced_input v0.23) support Bevy 0.18.
Both are designed for games with player entities, not design tools with a single global shortcut
context. leafwing-input-manager requires a monolithic Actionlike enum (violates plugin architecture
boundaries). bevy_enhanced_input's Unreal-style input context/modifier system is over-engineered for
keyboard shortcuts. Custom approach is ~200-300 lines, zero new deps, trivial command palette
integration, and no upgrade friction. **Alternatives rejected**: leafwing-input-manager
(entity-centric, monolithic action enum, 11K LOC for features we don't need), bevy_enhanced_input
(over-engineered, higher API churn, 22 breaking releases), bevy_input_actionmap (dormant, no Bevy
0.18), bevy-input-sequence (sequences only, no held-key support), action_maps (abandoned).

### 2026-02-16 — sublime_fuzzy chosen for palette search

**Context**: Fuzzy matching crate selection. **Decision**: `sublime_fuzzy` 0.7. **Rationale**:
Lightweight (zero-dependency), Sublime Text-style scoring, simple API (`best_match` returns
`Option<Match>` with `.score()`). `nucleo` considered but async-oriented and heavier than needed.

### 2026-02-16 — egui 0.33 API surprises

**Context**: Building palette UI. **Findings**: `screen_rect()` deprecated → use `content_rect()`.
`Frame::none()` deprecated → use `Frame::NONE`. `Margin::symmetric` takes `i8` not `f32`. Wiki
research page didn't cover these deprecations. Worth updating the bevy-egui guide.

### 2026-02-16 — No-op commands for discoverability

**Context**: Design doc targets ~25-30 commands, but many features (undo/redo, panel toggles) don't
exist yet. **Decision**: Register all planned commands as no-ops that log "not yet implemented."
**Rationale**: Palette feels comprehensive from day one. Users discover available shortcuts even for
features not yet built. Easy to wire up real handlers later.

## Test Results

### 2026-02-16 — 195 tests, zero clippy warnings

All 195 tests pass (13 config tests, 4 fuzzy search tests, 11 registry tests, plus all existing
tests). Zero clippy warnings. Clean boundary check.

## Blockers

| Blocker | Waiting On | Raised | Resolved |
| ------- | ---------- | ------ | -------- |
|         |            |        |          |

## Deferred / Future Work

- Backing implementations for remaining no-op commands: undo/redo (#84), select all (#108), grid
  overlay (#109), fullscreen (#110)

## Status Updates

| Date       | Status   | Notes                                                        |
| ---------- | -------- | ------------------------------------------------------------ |
| 2026-02-16 | speccing | Initial spec created, design doc reviewed                    |
| 2026-02-16 | speccing | Research spike complete — custom registry confirmed          |
| 2026-02-16 | building | Registry + persistence migration committed (fde6545)         |
| 2026-02-16 | building | Camera + hex_grid migration committed (14b4392)              |
| 2026-02-16 | building | Command palette UI + tool/mode shortcuts committed (ae3c5ca) |
| 2026-02-16 | building | TOML config + expanded command set (28 total) (84d4d56)      |
