# Changelog

All notable changes to Hexorder are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to
[Semantic Versioning](https://semver.org/).

## [0.14.0] — 2026-02-26

### Added

- add remote branch cleanup to ship and teardown checklists (project)
- add UAT criteria field to pitch template (project)
- add per-pitch UAT gate to workflow (project)
- document 95% coverage target in ship gate (project)
- add ADR directory and template (project)
- backfill foundational ADRs 001-004 (project)
- add task list ID to orientation checklist (project)

### Changed

- set up feature branch (project)
- add disk cleanup automation (project)
- set up feature branch (project)
- remove unnecessary bevy_window feature (contracts)

### Fixed

- use clone_from per clippy assigning_clones (map_gen)
- resolve typos and clippy warnings (project)

## [0.13.0] — 2026-02-25

### Added

- update docs for workspace structure (contracts)
- add has_new_records flag to UndoStack (ref #172) (undo_redo)
- add sync_dirty_flag system (ref #172) (persistence)
- add unsaved-changes confirmation dialog (ref #172) (persistence)
- update dirty flag documentation (ref #172) (persistence)
- add dirty flag tracking implementation plan (persistence)
- add undoable unit deletion (ref #127) (editor_ui)
- show dirty indicator in window title (ref #172) (persistence)
- add Scope 1+5 design document (settings)
- add Scope 1+5 implementation plan (settings)
- add settings contract types (contracts)
- add settings contract spec (contracts)
- add SettingsPlugin with three-layer merge (settings)
- update architecture and plugin log for Scope 1+5 (settings)
- migrate preferences to read from SettingsRegistry (settings)
- update plugin log for Scope 2 (settings)
- add custom theme loading and selection (settings)
- add keyboard shortcuts reference panel (settings)
- move reflection before ready for integration (project)
- add async dialog design and implementation plan (persistence)
- add async dialog infrastructure (persistence)
- log Scope 1 async dialog wrapper (persistence)
- add Scope 2 async dialog migration design (persistence)
- add dialog completion observer and dispatch (persistence)
- convert all trigger observers to async dialog (persistence)
- add save/load/dispatch tests (persistence)
- log Scope 2 async dialog migration (persistence)
- add Scope 2 async dialog migration plan (persistence)
- migrate export dialog to async folder picker (export)
- add save-point dirty tracking to UndoStack (undo_redo)
- integrate save-point API and migrate async dialogs (persistence)
- record undo commands for map generation (map_gen)

### Changed

- set up feature branch (contracts)
- extract hexorder-contracts workspace crate (contracts)
- replace crate::contracts with hexorder_contracts (contracts)
- update tooling for workspace structure (contracts)
- set up feature branch (editor_ui)
- extract do_save helper for reuse (ref #172) (persistence)
- set up feature branch (settings)
- update imports for contracts workspace split (settings)
- set up feature branch (persistence)
- add Clone derives and then field to dialog types (persistence)
- extract save_to_path and build helpers (persistence)
- extract load_from_path helper (persistence)
- extract reset, close, and save-dialog helpers (persistence)
- remove blocking dialog code and dead helpers (persistence)
- migrate export dialog from IoTaskPool to direct future (export)
- move hexorder-contracts into crates/ directory (project)
- remove transient plan documents (project)
- bump version to 0.13.0 (project)

### Fixed

- update imports for hexorder-contracts crate (persistence)
- correct typo — unparseable to unparsable (settings)
- resolve clippy and boundary violations (persistence)
- remove unused async dialog variants and field (persistence)
- trigger CloseProjectEvent on project close (editor_ui)
- collapse nested if per clippy collapsible_if (map_gen)

## [0.12.0] — 2026-02-23

### Added

- establish tool/game boundary in constitution (project)
- add hex-observe skill and update hex-retro integration (project)
- add editor dock polish design (editor_ui)
- wire Inspector tab to tile/unit inspectors (#144) (editor_ui)
- add Edit menu with dynamic undo/redo labels (#130) (editor_ui)
- persist font size across sessions (#128) (persistence)
- add catalog data model and plugin registration (mechanic_reference)
- populate catalog with 56 mechanics from survey (mechanic_reference)
- add browsable UI panel with contract types (mechanic_reference)
- add scaffolding templates for 6 mechanics (mechanic_reference)
- wire template application from mechanic reference panel (editor_ui)
- update spec, log, and add contract spec (mechanic_reference)
- add heightmap generation design document (map_gen)
- add scope 1 implementation plan (map_gen)
- add plugin skeleton with heightmap and biome modules (map_gen)
- add heightmap and biome table unit tests (map_gen)
- add full generation pipeline integration test (map_gen)
- update spec criteria and log with scope 1 results (map_gen)
- add scope 5 UI implementation plan (map_gen)
- add generation parameter UI panel (map_gen)
- update spec and log with scope 5 UI results (map_gen)
- hammer scopes 3-4, capture as issues #150-#152 (map_gen)
- capture all deferred items as issues #153-#155 (map_gen)
- reframe scopes per Tool/Game Boundary (map_gen)
- add hex-edge contract implementation plan (map_gen)
- add hex-edge types to hex-grid spec (contracts)
- add hex-edge types to hex_grid contract (contracts)
- update spec and log with scope 6 results (map_gen)
- add export plugin skeleton with ExportTarget trait (export)
- update architecture and plugin log for Scope 1 (export)
- add counter sheet PDF generation with printpdf (export)
- update plugin log for Scope 2 completion (export)
- add hex map PDF generation with terrain coloring (export)
- update plugin log for Scope 3 completion (export)
- wire PDF export to editor UI with save dialog (export)
- update plugin log — all scopes complete (export)
- add map_gen shared types for cross-plugin access (contracts)
- add map generator dock tab (editor_ui)
- add keyboard pan direction regression tests (camera)
- add debug_panic! macro (ref #168) (project)
- add status bar panel (ref #159) (editor_ui)
- add dock layout persistence (ref #160) (editor_ui)

### Changed

- set up feature branch (editor_ui)
- group EditorDockViewer into sub-structs (#146) (editor_ui)
- set up feature branch (mechanic_reference)
- set up feature branch (map_gen)
- add noise crate dependency (map_gen)
- extract default_layout helper in tests (map_gen)
- add ba to typos allowlist for hex edge tests (project)
- skip rand duplicate in deny.toml for noise crate (project)
- set up feature branch (export)
- set up feature branch (project)
- add dev profile build optimizations (ref #166) (project)
- add dbg_macro deny lint (ref #167) (project)
- fix taplo formatting in Cargo.toml (project)
- remove transient plan documents (project)
- bump version to 0.12.0 (project)
- generate changelog for v0.12.0 (project)

### Fixed

- integrate amplitude param and add biome table validation (map_gen)
- canonicalize HexEdge::new, init registry (contracts)
- replace game-specific biome defaults with neutral labels (map_gen)
- replace unwrap with expect in hex_grid tests (contracts)
- guard observer against missing HexGridConfig (export)
- load tiles without requiring EntityData (persistence)
- defer board load until tiles have EntityData (persistence)
- add assign_unit_visuals for loaded units without mesh (unit)
- reset viewport state on editor re-entry (camera)
- clear keyboard state after native file dialogs (persistence)
- disable egui zoom shortcuts to prevent Retina jitter (editor_ui)
- auto-scale hex map to fit page instead of rejecting (export)

## [0.11.0] — 2026-02-22

### Added

- add developer retrospective gate to hex-retro skill (project)
- add changelog header verification to ship workflow (project)
- clarify pre-release version source in branch setup (project)
- enforce worktree invariant for integration branches (project)
- add cargo safety guardrails to build workflow (project)
- add hex-bisect skill and shared target dir docs (project)
- add agent reflection protocol and checkpoints (project)
- strengthen abstraction check and add lines-changed (project)
- add test coverage enforcement with cargo-llvm-cov (project)
- add task list coordination protocol (#138) (project)
- add batch ceremonies and phase model (#139, #143) (project)
- log kickoff orientation for dockable panels (editor_ui)
- add egui_dock four-zone dockable layout (Scope 1) (editor_ui)
- native panels for four-zone layout (Scope 2) (editor_ui)
- decompose panel system into four zone systems (Scope 3) (editor_ui)
- add about panel render tests (Scope 3) (editor_ui)
- add Scope 4 tab support design (editor_ui)
- add Scope 4 tab support implementation plan (editor_ui)
- replace zone systems with DockArea tab support (editor_ui)
- add workspace presets with Cmd+1-4 switching (Scope 5) (editor_ui)
- persist workspace preset to project file (Scope 6) (editor_ui)
- log Scope 5-6 decisions and quality gate results (editor_ui)
- update bevy-egui input passthrough to no-absorb strategy (project)

### Changed

- set up feature branch for build discipline (project)
- commit Cargo.lock version bump (project)
- set up feature branch for dockable panels (project)
- bump version to 0.11.0 (project)
- generate changelog for v0.11.0 (project)

### Fixed

- recognize bare version tags in changelog generation (project)
- correct pre-release version to 0.11.0-build-discipline (project)
- account for viewport margins in fit scale and centering (camera)
- resolve ship gate manual check findings (project)
- disable absorb system to restore viewport interaction (editor_ui)

## [0.10.0] — 2026-02-20

### Added

- mark Cycle 4 integration branch as shipped (project)
- open Cycle 5 betting — Sharpen the Tools (0.10.0) (project)
- set delivery order for Cycle 5 pitches (project)
- add /hex-status skill for situational awareness (project)
- eliminate coordination.md — use GitHub-native tracking (project)
- add build checklist to pitch template and kickoff (project)
- add abstraction check step to build loop (project)
- add egui deprecation table and contribution protocol (project)
- add formatting baseline step to branch setup (project)
- document observer resource safety pattern (project)
- add skill recommendation step to build workflow (project)
- implement fullscreen toggle via Cmd+F (closes #110) (editor_ui)
- add toast notification system (ref #121) (editor_ui)
- add user-configurable font size (ref #121) (editor_ui)
- add multi-selection system (ref #121) (editor_ui)
- add grid overlay toggle (ref #121) (editor_ui)
- add About panel and Help menu (ref #121) (editor_ui)
- add viewport discoverability hints (ref #121) (editor_ui)
- add tests for all QoL scopes (ref #121) (editor_ui)
- update plugin log for 0.10.0 QoL scopes (editor_ui)
- add undo-redo contract spec (contracts)
- implement undo-redo contract types (contracts)
- add implementation plan for scope 1+2 (undo_redo)
- add UndoRedoPlugin with shortcuts and exclusive system (undo_redo)
- update spec and log after Scope 1+2 complete (undo_redo)
- wire paint_cell to undo stack with SetTerrainCommand (cell)
- update spec and log for Scope 3 completion (undo_redo)
- wire unit placement to undo stack with PlaceUnitCommand (unit)
- update spec and log for Scope 4 completion (undo_redo)
- add CompoundCommand for atomic multi-action undo (undo_redo)
- update spec, log, and contract for Scope 5 (undo_redo)
- mark UAT checklist as passed (undo_redo)
- add cycle coordination agents, skills, and ops guide (project)

### Changed

- add plan cleanup step and remove shipped plans (project)
- add rustfmt hook for auto-formatting on edit (project)
- set up feature branch (project)
- set up feature branch for fast builds (project)
- trim Bevy features to 3D-only subset (project)
- activate dynamic linking for dev builds (project)
- share target dir and add sccache across worktrees (project)
- fix TOML formatting in Cargo.toml (project)
- set up feature branch (undo_redo)
- fix debug check for manual Debug impls and taplo fmt (project)
- bump version to 0.10.0 (project)
- bump version to 0.10.0 (project)

### Fixed

- update hex-status for GitHub-native tracking (project)
- pin Cargo.lock to stable dependency resolution (project)
- preserve camera across screen transitions (persistence)
- address UAT feedback on QoL scopes (ref #121) (editor_ui)
- remove duplicate undo/redo shortcut registrations (editor_ui)

## [0.9.0] — 2026-02-18

### Added

- add build reflection check to hex-ship gate (project)
- add Cycle 4 design docs and update coordination (project)
- fix typo in visual polish design doc (project)
- switch branch naming from slash to hyphen separator (project)
- add worktree trust step to feature branch setup checklist (project)
- add BrandTheme and visual polish (editor_ui)
- add 0.9.0 visual polish success criteria (editor_ui)
- add workspace lifecycle and project naming (persistence)
- update spec with workspace lifecycle requirements (persistence)
- mark #53 workspace lifecycle as merged to integration (project)
- add CRT data model and resolution logic (#77) (rules_engine)
- add Play mode and mechanics resources (#77) (game_system)
- save/load mechanics resources (#77) (contracts)
- add Mechanics tab for turn structure and CRT (#77) (editor_ui)
- add Play mode toggle and turn tracker (#77) (editor_ui)
- update spec and log for 0.9.0 mechanics (#77) (rules_engine)
- wire up inline CRT outcome cell editing (#77) (editor_ui)
- add combat execution panel in Play mode (closes #104) (editor_ui)
- mark #77 as merged and design doc as historical (project)
- add storage provider abstraction (persistence)
- record research spike results (closes #25) (shortcuts)
- add shortcut registry and migrate persistence plugin (shortcuts)
- migrate camera and hex_grid shortcuts to registry (shortcuts)
- add command palette UI and tool/mode shortcuts (shortcuts)
- add TOML config overrides and expand command set to 28 (shortcuts)
- update architecture, contract spec, and plugin log (shortcuts)
- implement deferred commands and update egui guide (shortcuts)
- add dynamic viewport centering with deferred reset (camera)
- add debug inspector, viewport margins, close command (editor_ui)
- mark #80 as merged and integration branch as shipping (project)

### Changed

- close Cycle 3 in coordination (project)
- add project-scoped permission settings (project)
- place bets for Cycle 4 (project)
- auto-approve read-only tool permissions (project)
- set up feature branch (editor_ui)
- mark #54 as merged to integration branch (project)
- set up workspace feature branch (project)
- add persistence as valid commit scope (project)
- set up feature branch for core mechanics (#77) (project)
- add Thr abbreviation to typos allow list (project)
- set up feature branch (shortcuts)
- fix ship gate audit failures (shortcuts)
- add modifier-aware is_pressed to ShortcutRegistry (contracts)
- use registry is_pressed for keyboard pan (camera)
- migrate debug panel toggle to shortcut registry (editor_ui)
- fix ship gate clippy warnings and spec parity (contracts)
- merge 0.9.0-shortcuts into 0.9.0 (project)
- bump version to 0.9.0 (project)

### Fixed

- clean up indicator entities on editor exit (hex_grid)
- escape doc comment brackets to fix rustdoc warning (editor_ui)
- adapt play panel to workspace lifecycle API (editor_ui)
- center launcher Create/Cancel buttons at input width (editor_ui)
- register ShortcutsPlugin before consumers in load order (shortcuts)
- account for top menu bar in viewport centering (camera)
- prevent observer panic when SelectedHex missing (hex_grid)
- resolve post-merge clippy warnings and version (project)

## [0.8.0] — 2026-02-16

### Added

- worktree convention, lighter hooks, changelog fix (project)
- replace merge lock with integration branch model (project)
- add dependency sequencing for multi-pitch cycles (project)
- add UAT checklist to ship gate and spec template (project)
- address doc gaps #29, #30, #32, #36 (project)

### Changed

- release merge lock for 0.7.0/hex-grid-foundation (project)
- set up feature branch (project)
- bump version to 0.8.0 (project)

### Fixed

- regenerate changelog with v0.7.0 tag header (project)

## [0.7.0] — 2026-02-15

### Added

- require --ff-only for merges to main (project)
- add ontology editor UI panels (editor_ui)
- add UI architecture and test driver survey (project)
- add scripting and persistence plugins (M4.5 + M5) (project)
- adopt Shape Up methodology for development process (project)
- consolidate .specs/ into docs/ with single ownership (project)
- remove -guide suffix from guide filenames (project)
- remove roadmap.md and clean up references (project)
- remove domain.md, capture ideas as GitHub Issues (project)
- migrate research to GitHub Wiki with skill and guide (project)
- fix research skill frontmatter and remove cached lookups (project)
- deduplicate research skill against research guide (project)
- add routing step to research skill (project)
- add contract guide and skill (project)
- rename contracts skill to contract (singular) (project)
- add feature guide and skill (project)
- enforce hyphenated markdown filenames (project)
- rename feature to plugin for code modules (project)
- add triage, bet, and kickoff skills (project)
- add hex-idea skill, namespace all skills with hex- prefix (project)
- add hex-commit skill for atomic commit workflow (project)
- add hex-skill for skill creation and maintenance (project)
- streamline hex-commit skill workflow (project)
- retrofit hex- skills for convention compliance (project)
- add hex-d2 skill for D2 diagram creation (project)
- add hex-plantuml skill for PlantUML and Salt wireframes (project)
- add ditaa support to hex-plantuml skill (project)
- add render directives and quick reference to hex-d2 skill (project)
- improve ditaa alignment guidance in hex-plantuml skill (project)
- add design doc and implementation plan for pitch #81 (game_system)
- add EnumRegistry and StructRegistry types (contracts)
- add 6 compound PropertyType and PropertyValue variants (contracts)
- insert EnumRegistry and StructRegistry at startup (game_system)
- add v2 persistence with enum and struct registries (contracts)
- add Enums and Structs editor tabs with full CRUD (editor_ui)
- extend property form and recursive value renderer (editor_ui)
- update specs and log for 0.7.0 property system (contracts)
- add hex grid foundation design document (hex_grid)
- add implementation plan for hex grid foundation (hex_grid)
- add LineOfSightResult and VisibilityRange types (contracts)
- add neighbors, ring, and hex_range algorithm wrappers (hex_grid)
- add line_of_sight algorithm (hex_grid)
- add field_of_view and find_path algorithms (hex_grid)
- add LOS ray gizmo visualization (hex_grid)
- add LOS system integration test (hex_grid)
- update spec and log for 0.7.0 algorithms and LOS (hex_grid)

### Changed

- migrate backlog to GitHub Issues + Projects (project)
- add missing CI checks to match local pre-commit (project)
- remove unnecessary gitleaks license secret (project)
- unify CI and local checks through mise tasks (project)
- remove premature macOS app bundle infrastructure (project)
- extract wiki skill from research skill (project)
- move filename check to cross-platform Rust test (project)
- use Rust architecture test for filename check (project)
- set up feature branch (game_system)
- extract enum_definitions to EnumRegistry (contracts)
- prepare 0.7.0 merge (project)
- set up feature branch (hex_grid)
- enable hexx algorithms feature (hex_grid)
- bump version to 0.7.0 and claim merge lock (project)
- bump version to 0.7.0 (project)

### Fixed

- replace removed taplo-action with install-action (project)

## [0.4.0] — 2026-02-13

### Added

- document new project tooling in README (project)
- reference mise tasks from workflow docs (project)
- simplify getting started to 3 steps (project)
- use mise shorthand instead of mise run (project)
- add repo URL to getting started clone command (project)
- spec M4 milestone — rules shape the world (project)
- add ontology framework and validation contracts (ontology)
- add BFS valid move computation (rules_engine)
- add move overlays and unit ValidMoveSet check (hex_grid)
- update M4 coordination and feature logs (project)

### Changed

- add prettier formatting support (project)
- format all markdown with prettier (project)
- add gitleaks for secret and PII detection (project)
- add rustfmt configuration (project)
- format rust code with rustfmt (project)
- add clippy and rustc lint configuration (project)
- add bevy dynamic linking and release profile (project)
- add cargo-deny dependency auditing (project)
- add editorconfig (project)
- add typos spell checker (project)
- add taplo TOML formatter (project)
- format TOML files with taplo (project)
- add bacon watch mode configuration (project)
- add mise check and fix task hierarchy (project)
- add cargo fmt check to pre-commit hook (project)
- add GitHub Actions CI workflow (project)
- add boundary and unwrap check tasks (project)
- add audit, changelog, setup, and handoff tasks (project)
- declare proprietary license (project)
- acknowledge known duplicate dependencies (project)
- add prettier, gitleaks, debug, and doc checks (project)
- format files with prettier (project)
- run mise check in pre-commit hook (project)
- add fix:clippy task for auto-fixable lints (project)
- set up entity unification feature branch (project)
- unify entity types with EntityRole (game_system)
- bump version to 0.4.0 (project)
- bump version to 0.4.0 (project)

### Fixed

- fix cargo-deny and taplo check failures (project)

## [0.3.0] — 2026-02-11

### Added

- add M1-M3 codebase (project)
- add development process and tooling (project)

### Changed

- bump version to 0.3.0 (project)
