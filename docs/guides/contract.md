# Hexorder — Contracts Guide

## Purpose

Contracts define shared interfaces between plugins. They are the **only** coupling points between
plugins — all cross-plugin communication flows through contract types. This keeps plugins
independently buildable and testable.

## Where Contracts Live

Every contract has two mirrored locations:

- **Spec**: `docs/contracts/<name>.md` — the authoritative type definitions, written first
- **Code**: `crates/hexorder-contracts/src/<name>.rs` — the implementation, must match the spec

These must stay in parity. The ship gate verifies this: every type in
`crates/hexorder-contracts/src/` has a matching spec in `docs/contracts/`, and vice versa.

The spec template is in the [Spec Template](#spec-template) section below.

## What Contracts Contain

Contracts expose **data types only** — no systems, no logic:

- Components (`#[derive(Component)]`)
- Resources (`#[derive(Resource)]`)
- Events (`#[derive(Event)]`)
- Enums and utility types

### Conventions

- All types: `#[derive(Debug, Clone)]` minimum
- All fields: `pub` (contracts are shared interfaces)
- Components add `#[derive(Component)]`
- Resources add `#[derive(Resource)]`
- Events add `#[derive(Event)]`

## When to Create or Change a Contract

- **New contract**: When a plugin needs to expose types consumed by other plugins
- **Change contract**: When shared types need new fields, variants, or behavior
- **No contract needed**: When types are internal to a single plugin

Check `docs/architecture.md` for the plugin dependency graph to understand which plugins consume
which contracts.

## The Protocol

### Adding a New Contract

1. Write the spec at `docs/contracts/<name>.md` using the template
2. Create a GitHub Issue describing the addition with the `area:contracts` label
3. Implement the Rust types in `crates/hexorder-contracts/src/<name>.rs`
4. Register the module in `crates/hexorder-contracts/src/lib.rs`
5. Run `cargo build` to verify all consumers compile
6. Update `docs/architecture.md` dependency graph if needed

### Changing an Existing Contract

1. Create a GitHub Issue describing the change with the `area:contracts` label
2. Update the spec in `docs/contracts/<name>.md`
3. Update the implementation in `crates/hexorder-contracts/src/<name>.rs`
4. Run `cargo build` to verify all consumers still compile
5. Notify affected plugins (check `docs/architecture.md` for the dependency graph)

### Multi-Agent Coordination

Contract changes affect multiple plugins. When working in parallel:

- Before touching a contract, check for pending changes:
  `gh issue list --label "area:contracts" --state open`
- After changing a contract, run `cargo build` to catch breakage
- Contract changes should always be done solo, not in parallel

### Conflict Resolution

If two pitches propose incompatible changes to the same contract:

1. **First to merge wins.** The first pitch to land its contract change in the integration branch
   sets the shape of the contract.
2. **Second must adapt.** The other pitch rebases onto the integration branch and adjusts its
   implementation to work with the new contract shape.
3. **Neither merged yet?** Coordinate via a contract-specific GitHub Issue. Both agents propose
   their changes, the user decides the final shape, and both pitches adapt.

## Spec Template

Use this template when creating a new contract spec at `docs/contracts/<name>.md`:

````markdown
# Contract: [NAME]

## Purpose

[One sentence: what shared interface does this contract define?]

## Consumers

- [Plugin that reads/uses these types]

## Producers

- [Plugin that creates/writes these types]

## Types

### Components

```rust
#[derive(Component, Debug, Clone)]
pub struct ExampleComponent {
    pub field: Type,
}
```

### Resources

```rust
#[derive(Resource, Debug)]
pub struct ExampleResource {
    pub field: Type,
}
```

### Events

```rust
#[derive(Event, Debug)]
pub struct ExampleEvent {
    pub field: Type,
}
```

### Enums / Utility Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExampleEnum {
    VariantA,
    VariantB,
}
```

## Invariants

- [What must always be true about these types?]

## Changelog

| Date | Change             | Reason |
| ---- | ------------------ | ------ |
|      | Initial definition |        |
````
