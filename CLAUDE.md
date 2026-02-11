# Hexorder — Agent Instructions

## Project

Hexorder is a **game system design tool** built with Bevy 0.18 and Rust (edition 2024). It is NOT a
consumer game — it is a 3D turn-based hex simulation workbench for designing tabletop war board
games set in historical settings.

**Primary purpose**: Enable users to define rules, develop aesthetics, run experiments, and save
game system definitions for future distribution. A separate application will consume the exported
game system assets.

**Primary platform**: macOS (additional platforms later).

**Key distinction**: Hexorder is a design tool and simulator, not the final game product.

## Before You Start ANY Work

1. Follow the **Getting Started** section in `README.md` — prerequisites, tool installation, build
   verification
2. Read `.specs/constitution.md` — non-negotiable project rules
3. Read `.specs/coordination.md` — active features, ownership, cross-cutting concerns
4. Read `docs/git-guide.md` — git workflow, branching, commit, and merge conventions
5. Read `docs/bevy-guide.md` — Bevy 0.18 API reference, patterns, and pitfalls
6. Read `docs/bevy-egui-guide.md` — bevy_egui 0.39 API reference (if working on UI features)
7. Read the relevant `.specs/features/<name>/spec.md` for your assigned feature
8. Read `.specs/contracts/` for any shared types your feature depends on or exposes
9. Check `.specs/features/<name>/log.md` for prior decisions and blockers

## Architecture Rules

- Every feature is a Bevy Plugin in its own module under `src/`
- Shared types live in `src/contracts/` and are mirrored in `.specs/contracts/`
- Use Events for cross-feature communication, never direct coupling
- Components, Resources, Events must derive standard Bevy traits + Debug
- Prefer systems over methods; prefer queries over direct world access
- All public API types go through contracts first (spec before code)

## Bevy 0.18 Conventions

> Full reference: `docs/bevy-guide.md` — covers API, patterns, testing, migration notes, and
> pitfalls. egui reference: `docs/bevy-egui-guide.md` — covers bevy_egui 0.39 setup, scheduling,
> widgets, input passthrough, and styling.

- App builder: `app.add_plugins(MyPlugin)`
- Systems: `add_systems(Update, my_system)` with explicit schedule labels
- Camera: `Camera3d` component (this is a 3D application)
- Use `#[derive(Component)]`, `#[derive(Resource)]` for components and resources
- **Observer events** (immediate, trigger-based): `#[derive(Event)]`, fire with
  `commands.trigger()`, observe with `app.add_observer()`
- **Buffered messages** (pull-based, double-buffered): `#[derive(Message)]`, register with
  `app.add_message::<M>()`, use `MessageWriter<M>` / `MessageReader<M>`
- `EventReader`/`EventWriter`/`app.add_event` are **deprecated** — use the patterns above
- Use `Res<T>` / `ResMut<T>` for resources, `Query<>` for components
- States: `app.init_state::<GameState>()`, `in_state(GameState::Playing)`
- System ordering: use `.chain()` on tuples or `SystemSet` — `.after(bare_fn)` does not compile

## Development Workflow

> Git reference: `docs/git-guide.md` — branching, worktrees, commit format, merge checklists.

1. **Branch**: Run the Feature Branch Setup Checklist in `docs/git-guide.md` — creates branch,
   worktree, pre-release version, spec scaffolding, and claims ownership
2. **Spec first**: Read/update `.specs/features/<name>/spec.md` before coding
3. **Contract check**: If your feature exposes or consumes shared types, check `.specs/contracts/`
4. **Implement**: Write the plugin, systems, components in `src/<feature_name>/`
5. **Test**: Run `mise run check` (or individually: `cargo test`, `cargo clippy --all-targets`);
   update spec success criteria
6. **Commit**: Follow the Pre-Commit Checklist in `docs/git-guide.md` — commit early and often on
   the feature branch
7. **Boundary check**: Run `mise run check:boundary` — verifies no cross-feature internal imports.
   All shared types must go through `src/contracts/`
8. **Log**: Record decisions, test results, blockers in `.specs/features/<name>/log.md`
9. **Coordinate**: Update `.specs/coordination.md` status when starting/finishing work
10. **Merge**: When the feature is complete, follow the Pre-Merge Checklist in `docs/git-guide.md` —
    version bump, changelog, tag
11. **Teardown**: After merge is verified, run the Feature Branch Teardown Checklist in
    `docs/git-guide.md` — remove worktree, delete branch, update ownership

## Milestone Completion Gate

Before a milestone is marked complete, run a **constitution audit** across the full codebase.

**Automated checks** — run `mise run check:audit` to verify all of these at once:

1. `cargo test` — all tests pass
2. `cargo clippy --all-targets` — zero warnings (pedantic lints via `[lints.clippy]` in Cargo.toml)
3. `cargo build` — clean compilation
4. **No `unwrap()` in production code** (test files exempt) — `mise run check:unwrap`
5. **No cross-feature internal imports** — `mise run check:boundary`
6. Formatting, typos, TOML, dependency audit — all covered by `mise run check`

**Manual checks** — these require human judgment and cannot be automated:

7. **No `unsafe` without documented justification**
8. **All public types derive `Debug`**
9. **Contracts spec-code parity** — every type in `src/contracts/` has a matching spec in
   `.specs/contracts/`, and vice versa
10. **Brand palette compliance** — the `editor_ui_colors_match_brand_palette` architecture test
    passes. Any new color literals in `src/editor_ui/` must be added to the approved palette in the
    test and documented in `.specs/brand.md`
11. Record audit results in `.specs/coordination.md` under "Integration Test Checkpoints"

This gate applies even if all individual features pass their own success criteria. Constitution
violations that only emerge at the cross-feature level (like import boundary violations) are caught
here.

After the gate passes, follow the "Milestone final merge" steps in `docs/git-guide.md` — tag the
milestone version and record it in coordination.md.

## Testing Commands

- `mise run check` — run all checks (fmt, clippy, test, deny, typos, taplo, boundary, unwrap)
- `mise run check:audit` — full constitution audit (same as `check`, used at milestone gates)
- `cargo test` — all unit and integration tests
- `cargo clippy --all-targets` — lint check (pedantic, configured in Cargo.toml)
- `cargo test --lib <feature_name>` — feature-specific tests
- `mise run check:boundary` — cross-feature import boundary check
- `mise run check:unwrap` — no unwrap() in production code

## Shared Contracts Protocol

When you need to ADD or CHANGE a contract:

1. Propose the change in `.specs/coordination.md` under "Pending Contract Changes"
2. Update the spec in `.specs/contracts/<name>.md`
3. Implement the Rust types in `src/contracts/<name>.rs`
4. Run `cargo build` to verify all consumers still compile
5. Notify affected features (check coordination.md for dependencies)

## Agent Coordination Model

### Solo (simple feature, no dependencies)

Work through the development workflow above.

### Agent Teams (complex feature, internal parallelism)

Lead decomposes the feature into subtasks. Teammates each own a subsystem.

- Lead: owns the spec, decomposes work, reviews integration
- Teammates: own individual systems/components within the feature plugin

### Multi-Terminal (across-feature parallelism)

Multiple Claude Code sessions share a task list via `CLAUDE_CODE_TASK_LIST_ID`.

- Each terminal owns one feature in its own git worktree and branch (see `docs/git-guide.md`)
- Coordination happens through `.specs/coordination.md` and contracts
- Before touching a contract, check coordination.md for pending changes
- After changing a contract, run `cargo build` to catch breakage
- **Before merging to `main`**: claim the Merge Lock in `.specs/coordination.md` — only one merge at
  a time (see `docs/git-guide.md` → Merge Lock Protocol)
- Merges to `main` follow the Pre-Merge Checklist in `docs/git-guide.md`

## When to Spawn Teammates vs Work Solo

- **Solo**: feature has < 3 systems, no internal parallelism opportunity
- **Team**: feature has 3+ independent subsystems
- **Always solo for**: contract changes, cross-feature integration testing

## File Organization

```
src/
  main.rs              # App setup, plugin registration
  contracts/           # Shared types (mirrors .specs/contracts/)
    mod.rs
  <feature_name>/
    mod.rs             # Plugin definition
    components.rs      # Feature-local components
    systems.rs         # Systems
    events.rs          # Feature-local events
    tests.rs           # Unit tests (#[cfg(test)])
```
