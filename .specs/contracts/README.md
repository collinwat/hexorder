# Contracts

Contracts define shared interfaces between features. They are the ONLY coupling
points between feature plugins.

## Rules
1. Every contract has a spec (`.specs/contracts/<name>.md`) and an implementation (`src/contracts/<name>.rs`)
2. The spec is written FIRST. The implementation must match the spec.
3. Changing a contract requires updating coordination.md "Pending Contract Changes"
4. After changing a contract, `cargo build` must pass (all consumers must still compile)
5. Contracts expose: Components, Resources, Events, and utility types/functions
6. Contracts must NOT contain systems or logic — only data types and traits

## Convention
- Types use `#[derive(Debug, Clone)]` minimum
- Components add `#[derive(Component)]`
- Resources add `#[derive(Resource)]`
- Events add `#[derive(Event)]`
- All fields are `pub` (contracts are shared interfaces)
