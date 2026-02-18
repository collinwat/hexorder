# Changelog

All notable changes to Hexorder are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to
[Semantic Versioning](https://semver.org/).

## [Unreleased]

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
