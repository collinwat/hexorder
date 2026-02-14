# Hexorder

A **game system design tool** built with Bevy 0.18 and Rust. Hexorder is a 3D turn-based hex
simulation workbench for designing tabletop war board games set in historical settings.

Hexorder is not a consumer game — it is a design tool and simulator. Users define rules, develop
aesthetics, run experiments, and export game system definitions. A separate application consumes the
exported assets for distribution.

## Prerequisites

- **Rust** (stable, edition 2024) — install via [rustup](https://rustup.rs/)
- **mise** — install via [mise.jdx.dev](https://mise.jdx.dev/getting-started.html)

## Getting Started

```bash
# Clone and enter the repository
git clone git@github.com:collinwat/hexorder.git hexorder
cd hexorder

# Install tools, configure git hooks and LFS
mise setup

# Verify everything works
cargo test

# Run the application (use --features dev for faster rebuilds)
cargo run
```

## Project Structure

```
src/
  main.rs              # App setup, plugin registration
  contracts/           # Shared types across features
  camera/              # Orthographic top-down camera (pan, zoom)
  hex_grid/            # Hex grid rendering, tile selection, hover
  game_system/         # Game System container, type registries, properties
  cell/                # Cell painting and visual sync
  unit/                # Unit placement, movement, deletion
  editor_ui/           # Editor panels, tools, inspector

docs/
  constitution.md      # Non-negotiable project rules
  coordination.md      # Active cycle, ownership, merge lock
  architecture.md      # Plugin load order, cross-cutting concerns, dependency graph
  domain.md            # Product domain model
  brand.md             # Visual identity (colors, typography, icon)
  glossary.md          # Canonical terminology
  contracts/           # Shared type specifications
  features/            # Per-feature specs and logs
  guides/
    git.md       # Git workflow, branching, commit, merge conventions
    bevy.md      # Bevy 0.18 API reference and patterns
    bevy-egui.md # bevy_egui 0.39 API reference

# Project root config
CLAUDE.md              # Agent workflow and architecture rules
Cargo.toml             # Rust package manifest, lint configuration
mise.toml              # Project tools and task definitions
lefthook.yml           # Git hook definitions (fmt, build, secrets)
.github/workflows/     # CI pipeline (fmt, clippy, test, deny, typos, taplo)
```

## Development

Every feature is a Bevy Plugin in its own module under `src/`. Shared types live in `src/contracts/`
and are specified in `docs/contracts/`. Cross-feature communication uses Events only.

### Common commands

| Command                      | Purpose                                                           |
| ---------------------------- | ----------------------------------------------------------------- |
| `cargo build`                | Compile the project                                               |
| `cargo test`                 | Run all unit and integration tests                                |
| `cargo clippy --all-targets` | Lint check (pedantic, must pass with zero warnings)               |
| `cargo run`                  | Launch the application                                            |
| `cargo run --features dev`   | Launch with dynamic linking (faster rebuilds)                     |
| `mise fix`                   | Run all fixers (fmt, taplo, prettier, typos)                      |
| `mise check`                 | Run all checks (fmt, clippy, test, deny, typos, boundary, unwrap) |
| `mise check:audit`           | Full constitution audit (pre-merge / release)                     |
| `mise changelog`             | Preview unreleased changelog                                      |
| `mise handoff`               | Session handoff — dump context for resuming work                  |
| `mise setup`                 | First-time project setup (tools, hooks, LFS)                      |
| `bacon`                      | Watch mode — continuous check/clippy/test                         |

### Git workflow

This project uses trunk-based development with git worktrees, operating within Shape Up build
cycles. See `docs/guides/git.md` for the full workflow and `docs/guides/shape-up.md` for the
methodology reference. The workflow includes:

- Branch naming and worktree setup
- Conventional commit message format
- Pre-commit and pre-merge checklists
- Merge lock protocol for parallel sessions
- Versioning and changelog generation

### Key tools

| Tool                                                      | Purpose                                        | Config           |
| --------------------------------------------------------- | ---------------------------------------------- | ---------------- |
| [mise](https://mise.jdx.dev/)                             | Project tool manager and task runner           | `mise.toml`      |
| [lefthook](https://github.com/evilmartians/lefthook)      | Git hooks (fmt check, build, secrets)          | `lefthook.yml`   |
| [git-lfs](https://git-lfs.com/)                           | Large file storage for binary assets           | `.gitattributes` |
| [git-cliff](https://git-cliff.org/)                       | Changelog generation from conventional commits | `cliff.toml`     |
| [prettier](https://prettier.io/)                          | Markdown formatter                             | `.prettierrc`    |
| [taplo](https://taplo.tamasfe.dev/)                       | TOML formatter                                 | `taplo.toml`     |
| [typos](https://crates.io/crates/typos-cli)               | Source code spell checker                      | `_typos.toml`    |
| [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) | Dependency audit (vulnerabilities, licenses)   | `deny.toml`      |
| [bacon](https://crates.io/crates/bacon)                   | Background code checker (watch mode)           | `bacon.toml`     |

### Code quality

| Layer           | What it enforces                                    | When it runs      |
| --------------- | --------------------------------------------------- | ----------------- |
| `rustfmt`       | Rust formatting (100-char width, Unix line endings) | Pre-commit hook   |
| `clippy`        | Pedantic lints with Bevy-specific overrides         | CI, `mise check`  |
| `cargo test`    | 71 unit, integration, and architecture tests        | CI, `mise check`  |
| `cargo-deny`    | Vulnerability, license, and source auditing         | CI, `mise check`  |
| `typos`         | Spell checking across code and docs                 | CI, `mise check`  |
| `taplo`         | TOML file formatting                                | CI, `mise check`  |
| `prettier`      | Markdown formatting (100-char width)                | `mise fix`        |
| `.editorconfig` | Cross-editor indent, charset, line ending defaults  | Editor-level      |
| GitHub Actions  | All of the above, automated on push/PR              | Push to main, PRs |

## Contributing

1. Read `CLAUDE.md` for agent workflow and architecture rules
2. Read `docs/constitution.md` for non-negotiable project rules
3. Read `docs/guides/git.md` for git conventions
4. Check `docs/coordination.md` for active cycle and ownership
5. Check `docs/architecture.md` for cross-cutting concerns and dependencies

## Platform

Primary target: **macOS**. Additional platforms will be added later.

## License

Proprietary. All rights reserved.
