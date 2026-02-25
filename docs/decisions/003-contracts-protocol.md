# ADR-003: Contracts Protocol

**Status:** accepted\
**Date:** 2026-02-25

## Context

Plugins need shared types (components, resources, events) to communicate. Without a formal protocol,
shared types could live anywhere — in the producing plugin, in a shared `src/contracts/` directory,
or duplicated across consumers. The project needed a single source of truth for shared interfaces
with spec-code parity enforcement.

## Decision

Shared types live in a dedicated crate with mirrored documentation:

1. **Spec first**: Write the type definition in `docs/contracts/<name>.md` before implementing it.
   The spec uses the contract template from `docs/guides/contract.md`.

2. **Code mirrors spec**: Implement the types in `crates/hexorder-contracts/src/<name>.rs`. Every
   public type in the code must appear in the spec, and vice versa.

3. **Module re-export**: Register the module in `crates/hexorder-contracts/src/lib.rs` as
   `pub mod <name>`.

4. **Plugin imports**: All plugins import shared types via `hexorder_contracts::<module>::<Type>`.
   Never via `crate::` paths to another plugin's internals.

5. **Parity enforcement**: The ship gate includes a manual check that every type in
   `crates/hexorder-contracts/src/` has a matching spec in `docs/contracts/`, and vice versa. The
   boundary check (`mise check:boundary`) enforces import discipline at CI time.

6. **Change protocol**: Contract changes require a GitHub Issue with the `area:contracts` label,
   spec update, code update, and `cargo build` verification that all consumers compile.

**Parity audit** (0.14.0): 13 active contract modules, 13 matching specs, 130+ shared types. Two
documented exceptions: `cell.md` (redirects to game_system) and `terrain.md` (retired, historical).
Zero spec orphans. Zero undocumented public types.

## Consequences

- Plugins depend on `hexorder-contracts` (a library crate), not on each other — Cargo enforces this
  at compile time
- Spec-first workflow catches type design issues before implementation
- New agents can understand the shared interface by reading specs without reading source code
- Contract changes are high-ceremony by design — they affect multiple plugins and require
  coordination (see `docs/guides/contract.md`)
- The separate crate enables parallel compilation: `hexorder-contracts` builds once, then all
  plugins build in parallel against it
