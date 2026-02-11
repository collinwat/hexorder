# Hexorder

A **game system design tool** built with Bevy 0.18 and Rust. Hexorder is a 3D turn-based hex simulation workbench for designing tabletop war board games set in historical settings.

Hexorder is not a consumer game — it is a design tool and simulator. Users define rules, develop aesthetics, run experiments, and export game system definitions. A separate application consumes the exported assets for distribution.

## Prerequisites

- **Rust** (stable, edition 2024) — install via [rustup](https://rustup.rs/)
- **mise** — install via [mise.jdx.dev](https://mise.jdx.dev/getting-started.html)

## Getting Started

```bash
# Clone the repository
git clone <repo-url> hexorder
cd hexorder

# Install project tools (lefthook, git-lfs, git-cliff)
mise install

# Configure git hooks and LFS
git lfs install
lefthook install

# Build
cargo build

# Run tests
cargo test

# Run the application
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

.specs/
  constitution.md      # Non-negotiable project rules
  coordination.md      # Active features, ownership, merge lock
  roadmap.md           # Milestone plan and checkpoint history
  contracts/           # Shared type specifications
  features/            # Per-feature specs and logs

docs/
  git-guide.md         # Git workflow, branching, commit, merge conventions
  bevy-guide.md        # Bevy 0.18 API reference and patterns
  bevy-egui-guide.md   # bevy_egui 0.39 API reference

# Project root config
CLAUDE.md              # Agent workflow and architecture rules
mise.toml              # Project tool declarations (lefthook, git-lfs, git-cliff)
lefthook.yml           # Git hook definitions
cliff.toml             # Changelog generation config
Cargo.toml             # Rust package manifest
```

## Development

Every feature is a Bevy Plugin in its own module under `src/`. Shared types live in `src/contracts/` and are specified in `.specs/contracts/`. Cross-feature communication uses Events only.

### Common commands

| Command | Purpose |
|---------|---------|
| `cargo build` | Compile the project |
| `cargo test` | Run all unit and integration tests |
| `cargo clippy -- -D warnings` | Lint check (must pass with zero warnings) |
| `cargo test --lib <feature>` | Run tests for a specific feature |
| `cargo run` | Launch the application |

### Git workflow

This project uses trunk-based development with git worktrees. See `docs/git-guide.md` for the full workflow, including:

- Branch naming and worktree setup
- Conventional commit message format
- Pre-commit and pre-merge checklists
- Merge lock protocol for parallel sessions
- Versioning and changelog generation

### Key tools

| Tool | Purpose | Config |
|------|---------|--------|
| [mise](https://mise.jdx.dev/) | Project tool manager | `mise.toml` |
| [lefthook](https://github.com/evilmartians/lefthook) | Git hooks (commit message validation, build check) | `lefthook.yml` |
| [git-lfs](https://git-lfs.com/) | Large file storage for binary assets | `.gitattributes` |
| [git-cliff](https://git-cliff.org/) | Changelog generation from conventional commits | `cliff.toml` |

## Contributing

1. Read `CLAUDE.md` for agent workflow and architecture rules
2. Read `.specs/constitution.md` for non-negotiable project rules
3. Read `docs/git-guide.md` for git conventions
4. Check `.specs/coordination.md` for active features and ownership

## Platform

Primary target: **macOS**. Additional platforms will be added later.

## License

TBD
