---
name: hex-contract
description:
    Create, modify, or consume shared contract types between plugins. Use when a plugin needs to
    expose or consume shared types, when adding cross-plugin communication, or when checking
    contract spec-code parity. Also use when the user invokes /hex-contract.
---

# Contract

For the full protocol, conventions, and rationale, see `docs/guides/contract.md`.

## Which Workflow?

1. Check `docs/contracts/` and `src/contracts/` for existing contracts relevant to your work
2. If you need to **consume** an existing contract → read the spec and use the types
3. If you need to **create** a new contract → follow the creation steps below
4. If you need to **change** an existing contract → follow the change steps below

## Creating a New Contract

1. Create `docs/contracts/<name>.md` using the template from `docs/guides/contract.md`
2. Propose the addition in `docs/coordination.md` under "Pending Contract Changes"
3. Implement the types in `src/contracts/<name>.rs` — data types only, no systems or logic
4. Register the module in `src/contracts/mod.rs`
5. Run `cargo build` to verify consumers compile
6. Update `docs/architecture.md` dependency graph if needed

## Changing an Existing Contract

1. Propose the change in `docs/coordination.md` under "Pending Contract Changes"
2. Update the spec in `docs/contracts/<name>.md`
3. Update the implementation in `src/contracts/<name>.rs`
4. Run `cargo build` to verify all consumers still compile
5. Check `docs/architecture.md` for affected features
