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
3. Check the active cycle: `gh issue list --milestone "<milestone>" --label "type:pitch"` — current
   bets, ownership (assignees), integration branch (milestone description)
4. Read `docs/architecture.md` — plugin load order, cross-cutting concerns, dependency graph
5. Read `docs/guides/git.md` — git workflow, branching, commit, and merge conventions
6. Read `docs/guides/bevy.md` — Bevy 0.18 API reference, patterns, and pitfalls
7. Read `docs/guides/bevy-egui.md` — bevy_egui 0.39 API reference (if working on UI features)
8. Read `docs/guides/plugin.md` — plugin spec and log lifecycle
9. Read `docs/guides/contract.md` — contract protocol and spec template (if exposing or consuming
   shared types)
10. Read `docs/guides/research.md` — research workflow and wiki consumption (if exploring unknowns)
11. Read `docs/glossary.md` — canonical terminology (use these terms in code, docs, and commits)
12. Read the relevant `docs/plugins/<name>/spec.md` for your assigned plugin
13. Read `docs/contracts/` for any shared types your plugin depends on or exposes
14. Check `docs/plugins/<name>/log.md` for prior decisions and blockers
15. Check GitHub Issues for the current release: `gh issue list --milestone "<milestone>"`

## Architecture Rules

- Every plugin is a Bevy Plugin in its own module under `src/`
- Shared types live in `src/contracts/` and are mirrored in `docs/contracts/`
- Use Events for cross-plugin communication, never direct coupling
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
- If you encounter an egui deprecation not in `docs/guides/bevy-egui.md` §16, add it to the
  deprecation table before committing your fix

## Development Workflow (Within a Build Cycle)

> This workflow applies during the **build phase** of a Shape Up cycle. Before this workflow starts,
> the pitch has been shaped, bet on, and assigned to a release.

### Getting Oriented (first 1-2 days)

When a cycle starts, do not jump straight into coding. Read the shaped pitch, explore the relevant
code, and think through the approach. This orientation period is normal and expected.

1. Read the pitch Issue for your assigned work
2. Read `docs/plugins/<name>/spec.md` and the pitch's solution sketch
3. Read `docs/contracts/` for any shared types your plugin depends on or exposes
4. Check `docs/plugins/<name>/log.md` for prior decisions and blockers
5. Explore relevant code paths and contracts
6. Identify the first piece to build end-to-end (see "Get One Piece Done" below)

### Get One Piece Done

Pick the most core, small, novel piece and build it end-to-end — working code and working tests — in
a few days. Vertical integration, not horizontal layers. This surfaces unknowns early.

### Build Loop

7. **Branch**: Run the Feature Branch Setup Checklist in `docs/guides/git.md` — creates branch,
   worktree, pre-release version, spec scaffolding, and claims ownership
8. **Spec first**: Read/update `docs/plugins/<name>/spec.md` before coding
9. **Contract check**: If your plugin exposes or consumes shared types, check `docs/contracts/`
10. **Implement**: Write the plugin, systems, components in `src/<plugin_name>/`
11. **Test**: Run `mise check` (or individually: `cargo test`, `cargo clippy --all-targets`); update
    spec success criteria. Run tests after each scope — do not batch tests across multiple scopes.
12. **Abstraction check**: Does this implementation hardcode something that could be a trait or
    interface? Would a small abstraction here prevent duplicate work in future scopes? If yes,
    refactor before committing. If uncertain, note it in the plugin log and move on.
13. **Commit**: Follow the Pre-Commit Checklist in `docs/guides/git.md` — commit early and often on
    the feature branch
14. **Reflect**: Post a scope completion comment on the pitch issue following the Reflection
    Protocol in `docs/guides/agent-ops.md`. Include lines changed and answer the reflection prompts.
15. **Boundary check**: Run `mise check:boundary` — verifies no cross-plugin internal imports. All
    shared types must go through `src/contracts/`
16. **Log**: Record decisions, test results, blockers in `docs/plugins/<name>/log.md`

### Progress Updates

Post comments on the pitch issue as you build. These comments are the agent's narrative of the build
— the retro will read them later. When completing a scope, reference the Build Checklist item number
and include the commit SHA. Post when something worth noting happens:

- A scope is completed — reference the checklist item number and commit SHA
- Something is harder or easier than the pitch anticipated
- A rabbit hole was encountered (or avoided thanks to the pitch calling it out)
- Scope was hammered — what was cut and why
- A dead end was explored before finding the right approach
- An assumption from the pitch turned out wrong

Keep updates concise — a few sentences, not an essay. The comment thread should read like a build
journal, not a status report.

```bash
gh issue comment <pitch-number> --body "Scope N complete (commit abc1234): <observations>"
```

### Scope Hammering

Continuously distinguish must-haves from nice-to-haves. Compare to the current baseline (what exists
today), not an imagined ideal. If time runs short, cut scope to ship — do not extend the cycle.

### Finishing

16. **Capture new ideas**: When you discover future work (tech debt, feature ideas, research needs,
    bugs), create a GitHub Issue. Search first (`gh issue list --search "<keywords>"`), then create
    with the appropriate template. Issues are raw idea capture, not commitments.
17. **Coordinate**: Claim ownership with `gh issue edit <n> --add-assignee @me` when starting work;
    close the issue when finishing (via closing keyword in commit or `gh issue close <n>`)
18. **Build reflection**: Post a final comment on the pitch issue summarizing the build experience.
    This is the agent's retrospective testimony — the `/hex-retro` skill will surface it later.
    Cover:
    - What was the final shape vs. the original pitch?
    - What was harder or easier than expected?
    - What would you do differently if building this scope again?
    - What did you learn that future agents (or future cycles) should know?
    - Were there repetitive multi-step workflows that would benefit from a dedicated skill? (A skill
      candidate needs 2+ use cases, involves non-deterministic decisions, and is expected to recur.)
19. **Merge**: When the scope is complete, follow the Pre-Merge Checklist in `docs/guides/git.md` —
    version bump, changelog, tag
20. **Teardown**: After merge is verified, run the Feature Branch Teardown Checklist in
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
5. **No cross-plugin internal imports** — `mise check:boundary`
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
    placeholders in specs, plugin logs, and source code have corresponding GitHub Issues. Search
    with `gh issue list --search "<keywords>"` to verify.

This gate applies even if all individual plugins pass their own success criteria. Constitution
violations that only emerge at the cross-plugin level (like import boundary violations) are caught
here.

After the gate passes, follow the appropriate merge workflow in `docs/guides/git.md` — Ship Merge
(integration branch → main) or Solo-Pitch Merge (feature branch → main directly). Tag the release
version.

### Gate Remediation

If the ship gate fails:

1. Fix violations on the feature branch (or integration branch)
2. Re-run `mise check:audit` — all checks must pass
3. Walk through manual checks again
4. Do **not** merge until every check passes

The gate is pass/fail — partial passes do not count. If the cycle deadline arrives and the gate
still fails, the circuit breaker fires (see below).

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
- `mise check:boundary` — cross-plugin import boundary check
- `mise check:unwrap` — no unwrap() in production code
- `mise fix` — run all auto-fixers (fmt, clippy, taplo, prettier, typos)
- `cargo test --lib <plugin_name>` — plugin-specific tests

## GitHub Issues Workflow

GitHub Issues serve two purposes in Shape Up:

1. **Raw idea capture** — personal tracker for observations, bugs, feature ideas, tech debt
2. **Shaped pitches** — formal proposals for the betting table (label: `type:pitch`)

Issues are NOT a prioritized backlog. They are a capture tool. Only shaped pitches drive work.

### Capturing Raw Ideas

Use `/hex-idea` to interactively capture a raw idea as a GitHub Issue. The skill walks through type
selection, optional brainstorming, template-guided questions, duplicate checking, and confirmation.

Agents also create issues programmatically for deferred items, bugs found during testing, and new
ideas. Always search before creating to avoid duplicates:

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

1. Create a GitHub Issue describing the change with the `area:contracts` label
2. Update the spec in `docs/contracts/<name>.md`
3. Implement the Rust types in `src/contracts/<name>.rs`
4. Run `cargo build` to verify all consumers still compile
5. Notify affected plugins (check `docs/architecture.md` for dependency graph)

## Agent Coordination Model

Agents are the "team" in Shape Up's building phase. The developer shapes and bets; agents build.

### Solo (simple plugin, no dependencies)

Work through the development workflow above.

### Agent Teams (complex plugin, internal parallelism)

Lead decomposes the plugin into subtasks. Teammates each own a subsystem.

- Lead: owns the spec, decomposes work, reviews integration
- Teammates: own individual systems/components within the plugin

### Multi-Terminal (across-plugin parallelism)

Multiple Claude Code sessions share a task list via `CLAUDE_CODE_TASK_LIST_ID`.

- Each terminal owns one plugin in its own git worktree and branch (see `docs/guides/git.md`)
- Coordination happens through GitHub Issues, milestones, and contracts
- Before touching a contract, check for pending changes:
  `gh issue list --label "area:contracts" --state open`
- After changing a contract, run `cargo build` to catch breakage
- **Feature branches merge to the integration branch** via Pitch Merge (see `docs/guides/git.md`)
- **Only Ship Merge touches `main`** — one merge per cycle, after the ship gate passes

## When to Spawn Teammates vs Work Solo

- **Solo**: plugin has < 3 systems, no internal parallelism opportunity
- **Team**: plugin has 3+ independent subsystems
- **Always solo for**: contract changes, cross-plugin integration testing

## File Organization

```
src/
  main.rs              # App setup, plugin registration
  contracts/           # Shared types (mirrors docs/contracts/)
    mod.rs
  <plugin_name>/
    mod.rs             # Plugin definition
    components.rs      # Plugin-local components
    systems.rs         # Systems
    events.rs          # Plugin-local events
    tests.rs           # Unit tests (#[cfg(test)])
```
