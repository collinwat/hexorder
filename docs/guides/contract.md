# Hexorder — Contracts Guide

## Purpose

Contracts define shared interfaces between feature plugins. They are the **only** coupling points
between features — all cross-feature communication flows through contract types. This keeps features
independently buildable and testable.

## Where Contracts Live

Every contract has two mirrored locations:

- **Spec**: `docs/contracts/<name>.md` — the authoritative type definitions, written first
- **Code**: `src/contracts/<name>.rs` — the implementation, must match the spec

These must stay in parity. The ship gate verifies this: every type in `src/contracts/` has a
matching spec in `docs/contracts/`, and vice versa.

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

- **New contract**: When a feature needs to expose types consumed by other features
- **Change contract**: When shared types need new fields, variants, or behavior
- **No contract needed**: When types are internal to a single feature plugin

Check `docs/architecture.md` for the feature dependency graph to understand which features consume
which contracts.

## The Protocol

### Adding a New Contract

1. Write the spec at `docs/contracts/<name>.md` using the template
2. Propose the addition in `docs/coordination.md` under "Pending Contract Changes"
3. Implement the Rust types in `src/contracts/<name>.rs`
4. Register the module in `src/contracts/mod.rs`
5. Run `cargo build` to verify all consumers compile
6. Update `docs/architecture.md` dependency graph if needed

### Changing an Existing Contract

1. Propose the change in `docs/coordination.md` under "Pending Contract Changes"
2. Update the spec in `docs/contracts/<name>.md`
3. Update the implementation in `src/contracts/<name>.rs`
4. Run `cargo build` to verify all consumers still compile
5. Notify affected features (check `docs/architecture.md` for the dependency graph)

### Multi-Agent Coordination

Contract changes affect multiple features. When working in parallel:

- Before touching a contract, check `docs/coordination.md` for pending changes
- After changing a contract, run `cargo build` to catch breakage
- Contract changes should always be done solo, not in parallel

## Spec Template

Use this template when creating a new contract spec at `docs/contracts/<name>.md`:

````markdown
# Contract: [NAME]

## Purpose

[One sentence: what shared interface does this contract define?]

## Consumers

- [Feature that reads/uses these types]

## Producers

- [Feature that creates/writes these types]

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
