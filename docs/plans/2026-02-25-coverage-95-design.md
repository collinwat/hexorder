# Coverage Improvement to 95% — Design

**Date**: 2026-02-25 **Baseline**: 50.98% line coverage (16,902 instrumented lines, 8,285 missed)
**Target**: 95% line coverage (need to additionally cover ~7,440 lines)

## Strategy: Tiered Bottom-Up with Feasibility Spike

Work quick wins first to build momentum and validate test patterns, then moderate gaps, then the
heavy lift. Parallelize independent modules within each tier. Brief feasibility spike on
`editor_ui/systems.rs` before committing to the full plan.

## Key Constraint

`editor_ui/systems.rs` is 6,211 instrumented lines at 13.9% coverage — it represents 64% of all
missed lines. Without covering this file, 95% overall is mathematically impossible. The project
already has `egui_kittest::Harness`-based tests in `ui_tests.rs` that validate this approach works.

## Phase 0 — Feasibility Spike

**Goal**: Confirm `editor_ui/systems.rs` can be systematically tested at scale.

- Review existing `ui_tests.rs` patterns
- Identify categories of uncovered code (rendering, event handling, state management)
- Write 2-3 representative tests covering different categories
- Estimate effort for full coverage
- **Go/no-go decision** before investing in Phases 1-3

## Phase 1 — Quick Wins (~325 missed lines → ~54% total)

Modules already above 80% or with small gaps. Low risk, high confidence.

| Workstream                     | Files                                                                                                                     | Missed Lines |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------- | ------------ |
| **A: contracts**               | contracts: game_system, hex_grid, undo_redo, persistence, mechanics, ontology, settings, shortcuts, editor_ui, validation | ~190         |
| **B: unit + cell + undo_redo** | unit/systems.rs, cell/systems.rs, undo_redo/systems.rs                                                                    | ~34          |
| **C: export**                  | export: counter_sheet, hex_map, mod.rs, systems.rs                                                                        | ~177         |
| **D: main + components**       | main.rs, editor_ui/components.rs, macros.rs                                                                               | ~133         |

**Execution**: Work through sequentially (A → B → C → D), committing after each.

## Phase 2 — Moderate Effort (~1,443 missed lines → ~62.5% total)

Modules at 40-80% coverage. Require understanding business logic to write meaningful tests.

| Workstream                       | Files                                                                                 | Missed Lines |
| -------------------------------- | ------------------------------------------------------------------------------------- | ------------ |
| **E: hex_grid + map_gen**        | hex_grid/systems.rs, hex_grid/mod.rs, map_gen: biome, systems, mod                    | ~414         |
| **F: persistence + scripting**   | persistence/systems.rs, persistence/async_dialog.rs, scripting: lua_api, mod, systems | ~560         |
| **G: ontology + rules_engine**   | ontology/systems.rs, rules_engine/systems.rs                                          | ~206         |
| **H: camera + shortcuts config** | camera/mod.rs, shortcuts/config.rs                                                    | ~169         |

**Execution**: Work through sequentially (E → F → G → H), committing after each.

## Phase 3 — Heavy Lift (~6,517 missed lines → ~95% total)

The biggest gaps. `editor_ui/systems.rs` alone is 5,346 missed lines.

| Workstream                        | Files                                                     | Missed Lines |
| --------------------------------- | --------------------------------------------------------- | ------------ |
| **I: editor_ui systems**          | editor_ui/systems.rs, editor_ui/mod.rs                    | ~5,634       |
| **J: camera + shortcuts systems** | camera/systems.rs, shortcuts/systems.rs, shortcuts/mod.rs | ~542         |
| **K: settings + remaining**       | settings: config, mod, systems; map_gen leftovers         | ~155         |

**Execution**: Work through sequentially (I → J → K), committing after each.

## Execution Model

- Sequential execution in a single session — no subagents or parallel worktrees
- Write tests in existing test files (`tests.rs`, `ui_tests.rs`) within each module
- No production code changes except where needed for testability (e.g., extracting pure functions)
- Coverage measurement after each phase to track progress and adjust
- User confirms before moving to the next phase

## Test Patterns

### Pure logic (contracts, rules_engine, shortcuts/config)

Direct unit tests — call function, assert result. No Bevy App needed.

### Bevy systems (most modules)

`App::new()` with minimal plugin setup, insert test resources/entities, run schedule, assert state.

### egui rendering (editor_ui)

`egui_kittest::Harness` to render UI elements and verify widget output.

### Plugin registration (0% mod.rs files)

Build a Bevy `App`, add the plugin, verify expected resources/systems exist.

### Async/IO (persistence, scripting)

Mock file system operations or use temp directories. Test serialization/deserialization logic
separately from actual I/O.

## Risk Assessment

| Risk                                          | Mitigation                                           |
| --------------------------------------------- | ---------------------------------------------------- |
| editor_ui/systems.rs is too hard to unit test | Phase 0 spike validates approach first               |
| Session context window fills up               | Commit after each task to create natural checkpoints |
| Tests are brittle or low-value                | Review tests for each phase before merging           |
| 95% requires covering Bevy boilerplate        | Integration-style tests with `App::new()`            |

## Progress Tracking

| Milestone     | Coverage | Delta  |
| ------------- | -------- | ------ |
| Baseline      | 50.98%   | —      |
| After Phase 1 | ~54%     | +3%    |
| After Phase 2 | ~62.5%   | +8.5%  |
| After Phase 3 | ~95%     | +32.5% |
