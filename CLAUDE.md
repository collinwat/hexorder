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
2. Read `docs/constitution.md` — non-negotiable project rules
3. Read `docs/coordination.md` — active cycle, ownership, merge lock
4. Read `docs/architecture.md` — plugin load order, cross-cutting concerns, dependency graph
5. Read `docs/guides/git.md` — git workflow, branching, commit, and merge conventions
6. Read `docs/guides/bevy.md` — Bevy 0.18 API reference, patterns, and pitfalls
7. Read `docs/guides/bevy-egui.md` — bevy_egui 0.39 API reference (if working on UI features)
8. Read `docs/guides/research.md` — research workflow and wiki consumption (if exploring unknowns)
9. Read the relevant `docs/features/<name>/spec.md` for your assigned feature
10. Read `docs/contracts/` for any shared types your feature depends on or exposes
11. Check `docs/features/<name>/log.md` for prior decisions and blockers
12. Check GitHub Issues for the current release: `gh issue list --milestone "<milestone>"`

## Architecture Rules

- Every feature is a Bevy Plugin in its own module under `src/`
- Shared types live in `src/contracts/` and are mirrored in `docs/contracts/`
- Use Events for cross-feature communication, never direct coupling
- Components, Resources, Events must derive standard Bevy traits + Debug
- Prefer systems over methods; prefer queries over direct world access
- All public API types go through contracts first (spec before code)

## Bevy 0.18 Conventions

> Full reference: `docs/guides/bevy.md` — covers API, patterns, testing, migration notes, and
> pitfalls. egui reference: `docs/guides/bevy-egui.md` — covers bevy_egui 0.39 setup, scheduling,
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

## Development Workflow (Within a Build Cycle)

> This workflow applies during the **build phase** of a Shape Up cycle. Before this workflow starts,
> the pitch has been shaped, bet on, and assigned to a release.

### Getting Oriented (first 1-2 days)

When a cycle starts, do not jump straight into coding. Read the shaped pitch, explore the relevant
code, and think through the approach. This orientation period is normal and expected.

1. Read the pitch Issue for your assigned work
2. Read `docs/features/<name>/spec.md` and the pitch's solution sketch
3. Read `docs/contracts/` for any shared types your feature depends on or exposes
4. Check `docs/features/<name>/log.md` for prior decisions and blockers
5. Explore relevant code paths and contracts
6. Identify the first piece to build end-to-end (see "Get One Piece Done" below)

### Get One Piece Done

Pick the most core, small, novel piece and build it end-to-end — working code and working tests — in
a few days. Vertical integration, not horizontal layers. This surfaces unknowns early.

### Build Loop

7. **Branch**: Run the Feature Branch Setup Checklist in `docs/guides/git.md` — creates branch,
   worktree, pre-release version, spec scaffolding, and claims ownership
8. **Spec first**: Read/update `docs/features/<name>/spec.md` before coding
9. **Contract check**: If your feature exposes or consumes shared types, check `docs/contracts/`
10. **Implement**: Write the plugin, systems, components in `src/<feature_name>/`
11. **Test**: Run `mise check` (or individually: `cargo test`, `cargo clippy --all-targets`); update
    spec success criteria
12. **Commit**: Follow the Pre-Commit Checklist in `docs/guides/git.md` — commit early and often on
    the feature branch
13. **Boundary check**: Run `mise check:boundary` — verifies no cross-feature internal imports. All
    shared types must go through `src/contracts/`
14. **Log**: Record decisions, test results, blockers in `docs/features/<name>/log.md`

### Scope Hammering

Continuously distinguish must-haves from nice-to-haves. Compare to the current baseline (what exists
today), not an imagined ideal. If time runs short, cut scope to ship — do not extend the cycle.

### Finishing

15. **Capture new ideas**: When you discover future work (tech debt, feature ideas, research needs,
    bugs), create a GitHub Issue. Search first (`gh issue list --search "<keywords>"`), then create
    with the appropriate template. Issues are raw idea capture, not commitments.
16. **Coordinate**: Update `docs/coordination.md` status when starting/finishing work
17. **Merge**: When the scope is complete, follow the Pre-Merge Checklist in `docs/guides/git.md` —
    version bump, changelog, tag
18. **Teardown**: After merge is verified, run the Feature Branch Teardown Checklist in
    `docs/guides/git.md` — remove worktree, delete branch, update ownership

## Ship Gate

Before a cycle's work ships, run a **constitution audit** across the full codebase. This is the
quality bar. If the cycle hits its deadline and the gate does not pass, the circuit breaker fires:
work does not ship, and the problem must be re-shaped and re-pitched.

**Automated checks** — run `mise check:audit` to verify all of these at once:

1. `cargo test` — all tests pass
2. `cargo clippy --all-targets` — zero warnings (pedantic lints via `[lints.clippy]` in Cargo.toml)
3. `cargo build` — clean compilation
4. **No `unwrap()` in production code** (test files exempt) — `mise check:unwrap`
5. **No cross-feature internal imports** — `mise check:boundary`
6. Formatting, typos, TOML, dependency audit — all covered by `mise check`

**Manual checks** — these require human judgment and cannot be automated:

7. **No `unsafe` without documented justification**
8. **All public types derive `Debug`**
9. **Contracts spec-code parity** — every type in `src/contracts/` has a matching spec in
   `docs/contracts/`, and vice versa
10. **Brand palette compliance** — the `editor_ui_colors_match_brand_palette` architecture test
    passes. Any new color literals in `src/editor_ui/` must be added to the approved palette in the
    test and documented in `docs/brand.md`
11. **No stray ideas** — all deferred scope, future work notes, TODOs, and "coming soon"
    placeholders in specs, feature logs, and source code have corresponding GitHub Issues. Search
    with `gh issue list --search "<keywords>"` to verify.

This gate applies even if all individual features pass their own success criteria. Constitution
violations that only emerge at the cross-feature level (like import boundary violations) are caught
here.

After the gate passes, follow the "Cycle ship merge" steps in `docs/guides/git.md` — tag the release
version and record it in coordination.md.

### Circuit Breaker

If a cycle does not finish by its deadline:

- The work is cancelled by default — it does not automatically roll into the next cycle
- The team re-shapes the problem, looking for a better approach
- A new pitch must be brought to the next betting table
- Extension is only granted if: (1) remaining tasks are true must-haves, and (2) all remaining work
  is downhill (no unsolved problems, pure execution)

## Testing Commands

- `mise test` — run all tests
- `mise test:cargo` — run Rust unit and integration tests
- `mise check` — run all checks (fmt, clippy, test, deny, typos, taplo, boundary, unwrap)
- `mise check:audit` — full constitution audit (same as `check`, used at release gates)
- `mise check:clippy` — lint check (pedantic, configured in Cargo.toml)
- `mise check:boundary` — cross-feature import boundary check
- `mise check:unwrap` — no unwrap() in production code
- `mise fix` — run all auto-fixers (fmt, clippy, taplo, prettier, typos)
- `cargo test --lib <feature_name>` — feature-specific tests

## GitHub Issues Workflow

GitHub Issues serve two purposes in Shape Up:

1. **Raw idea capture** — personal tracker for observations, bugs, feature ideas, tech debt
2. **Shaped pitches** — formal proposals for the betting table (label: `type:pitch`)

Issues are NOT a prioritized backlog. They are a capture tool. Only shaped pitches drive work.

### Capturing Raw Ideas

Agents create issues for deferred items, bugs found during testing, and new ideas. Always search
before creating to avoid duplicates:

```bash
gh issue list --search "<keywords>" --state all
```

Create with the appropriate template (feature, bug, tech-debt, research). New issues get
`status:triage` automatically:

```bash
gh issue create --title "<item>" --label "status:deferred" --label "type:<type>"
```

### Referencing Issues

- In commit messages: `feat(unit): add stacking support (fixes #42)`
- In spec Deferred Items: note the issue number
- In code comments: `// TODO(#42): implement stacking limit`

Closing keywords (`fixes`, `closes`, `resolves`) auto-close the issue when the PR/commit merges to
the default branch.

### Issue Lifecycle

1. **Captured** — `status:triage` label applied automatically. Issue is a raw idea.
2. **Triaged** — human assigns type/area labels, removes triage label
3. **Shaped** — during cool-down, promising ideas are shaped into pitch Issues (`type:pitch`)
4. **Bet** — pitch is selected at the betting table, assigned to a release milestone
5. **Claimed** — agent self-assigns when starting work within the cycle
6. **Closed** — via closing keyword in commit/PR, or `gh issue close <number>`

### Quick Reference

```bash
gh issue list --state open                     # all raw ideas
gh issue list --label "type:pitch"             # shaped pitches
gh issue list --label "type:pitch" -m "<rel>"  # pitches bet for a release
gh issue list --label "status:triage"          # items needing triage
gh issue list --search "<keywords>"            # search all issues
gh issue create                                # capture a raw idea
gh issue edit <n> --add-assignee @me           # claim a bet pitch
```

## Shared Contracts Protocol

When you need to ADD or CHANGE a contract:

1. Propose the change in `docs/coordination.md` under "Pending Contract Changes"
2. Update the spec in `docs/contracts/<name>.md`
3. Implement the Rust types in `src/contracts/<name>.rs`
4. Run `cargo build` to verify all consumers still compile
5. Notify affected features (check `docs/architecture.md` for dependency graph)

## Agent Coordination Model

Agents are the "team" in Shape Up's building phase. The developer shapes and bets; agents build.

### Solo (simple feature, no dependencies)

Work through the development workflow above.

### Agent Teams (complex feature, internal parallelism)

Lead decomposes the feature into subtasks. Teammates each own a subsystem.

- Lead: owns the spec, decomposes work, reviews integration
- Teammates: own individual systems/components within the feature plugin

### Multi-Terminal (across-feature parallelism)

Multiple Claude Code sessions share a task list via `CLAUDE_CODE_TASK_LIST_ID`.

- Each terminal owns one feature in its own git worktree and branch (see `docs/guides/git.md`)
- Coordination happens through `docs/coordination.md` and contracts
- Before touching a contract, check coordination.md for pending changes
- After changing a contract, run `cargo build` to catch breakage
- **Before merging to `main`**: claim the Merge Lock in `docs/coordination.md` — only one merge at a
  time (see `docs/guides/git.md` → Merge Lock Protocol)
- Merges to `main` follow the Pre-Merge Checklist in `docs/guides/git.md`

## When to Spawn Teammates vs Work Solo

- **Solo**: feature has < 3 systems, no internal parallelism opportunity
- **Team**: feature has 3+ independent subsystems
- **Always solo for**: contract changes, cross-feature integration testing

## File Organization

```
src/
  main.rs              # App setup, plugin registration
  contracts/           # Shared types (mirrors docs/contracts/)
    mod.rs
  <feature_name>/
    mod.rs             # Plugin definition
    components.rs      # Feature-local components
    systems.rs         # Systems
    events.rs          # Feature-local events
    tests.rs           # Unit tests (#[cfg(test)])
```
