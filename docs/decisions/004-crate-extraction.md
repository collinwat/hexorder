# ADR-004: Crate Extraction Pattern

**Status:** accepted\
**Date:** 2026-02-25

## Context

As the project grew, shared types in `src/contracts/` became a bottleneck: changes to any contract
file triggered recompilation of the entire binary. The workspace split (#174) extracted shared types
into a separate library crate (`hexorder-contracts`) to enable parallel compilation and
Cargo-enforced boundaries. This ADR documents when and how to extract crates, using
`hexorder-contracts` as the reference implementation.

## Decision

### When to extract a crate

Extract a module into a workspace crate when:

- The module defines types consumed by 3+ other modules (high fan-out)
- The module is stable enough that frequent changes are unlikely
- Compile-time isolation would measurably improve incremental build times
- Cargo-enforced boundaries are needed (preventing accidental internal imports)

Do **not** extract when:

- The module is consumed by only 1-2 other modules (low fan-out)
- The module is under active development with frequent API changes
- The extraction would require complex feature flag gymnastics

### How to extract

1. Create the crate under `crates/<name>/` with its own `Cargo.toml`
2. Add it to the workspace `members` list in the root `Cargo.toml`
3. Declare minimal dependencies — only what the crate's types actually need
4. Re-export all modules from `lib.rs`
5. Update consumers to depend on the new crate via `path = "crates/<name>"`
6. Verify with `cargo build` that all consumers compile
7. Document the crate in `docs/architecture.md`

### Dependency minimization

Extracted crates must declare only the Bevy features their types actually use. For
`hexorder-contracts` (audited 0.14.0):

| Feature          | Status    | Reason                                                |
| ---------------- | --------- | ----------------------------------------------------- |
| `bevy_state`     | Required  | `#[derive(States)]` on `AppScreen`                    |
| `3d_bevy_render` | Required  | `Handle<Mesh>`, `StandardMaterial`, `Transform`, etc. |
| `bevy_log`       | Removable | Not used by any contract type                         |
| `bevy_window`    | Removable | Not used by any contract type                         |

### Reference implementation

`crates/hexorder-contracts/` is the canonical example:

- **Workspace member**: Listed in root `Cargo.toml` workspace members
- **Dependency**: Root crate depends via
  `hexorder-contracts = { path = "crates/hexorder-contracts" }`
- **Bevy features**: `default-features = false` with explicit feature list
- **Additional deps**: `bevy_egui`, `hexx`, `ron`, `serde`, `uuid` — each justified by type usage
- **13 modules**: One per contract domain, all re-exported from `lib.rs`

## Consequences

- Parallel compilation: `hexorder-contracts` builds once, then all plugins build concurrently
- Cargo-enforced boundaries: A plugin cannot accidentally import another plugin's internals through
  the contracts crate
- Minimal features reduce the contracts crate's compile time and dependency tree
- Adding a new contract module requires updating both the crate and the spec (see ADR-003)
- Future crate extractions (e.g., a scripting API crate) should follow this same pattern
