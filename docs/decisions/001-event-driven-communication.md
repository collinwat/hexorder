# ADR-001: Event-Driven Cross-Plugin Communication

**Status:** accepted\
**Date:** 2026-02-25

## Context

Hexorder's plugin architecture requires plugins to communicate without direct coupling. Direct
function calls between plugins create compile-time dependencies, prevent independent testing, and
make plugin registration order fragile. The project needed a communication pattern that enforces
loose coupling while remaining type-safe.

Bevy 0.18 provides two event mechanisms: observer events (immediate, trigger-based) and buffered
messages (pull-based, double-buffered). The deprecated `EventReader`/`EventWriter` API also exists
but is being phased out.

## Decision

All cross-plugin communication uses **observer events** exclusively:

- Shared event types are defined in `crates/hexorder-contracts/src/` with `#[derive(Event)]`
- Producers fire events with `commands.trigger(MyEvent { ... })`
- Consumers register handlers with `app.add_observer(handler_fn)`
- No plugin imports another plugin's internal modules — all shared types flow through contracts
- The deprecated `EventReader`/`EventWriter` API must not be used

Buffered messages (`#[derive(Message)]`) are permitted but not currently used. If introduced,
message types must also live in contracts.

**Verified patterns** (as of 0.14.0):

| Event                  | Producer           | Consumer(s)                               |
| ---------------------- | ------------------ | ----------------------------------------- |
| `CommandExecutedEvent` | shortcuts          | camera, undo_redo, editor_ui, persistence |
| `HexSelectedEvent`     | hex_grid           | unit                                      |
| `HexMoveEvent`         | unit               | rules_engine                              |
| `UnitPlacedEvent`      | unit               | (available for observers)                 |
| `ToastEvent`           | any plugin         | editor_ui                                 |
| `SaveRequestEvent`     | editor_ui          | persistence                               |
| `LoadRequestEvent`     | editor_ui          | persistence                               |
| `NewProjectEvent`      | editor_ui          | persistence                               |
| `CloseProjectEvent`    | editor_ui          | persistence                               |
| `SettingsChanged`      | settings           | (available for observers)                 |
| `PhaseAdvancedEvent`   | mechanics (future) | (not yet consumed)                        |
| `CombatResolvedEvent`  | mechanics (future) | (not yet consumed)                        |

**Compliance audit** (0.14.0): Zero cross-plugin internal imports. Zero deprecated
`EventReader`/`EventWriter` usage. Zero `#[derive(Message)]` usage (observer-only pattern).

## Consequences

- Plugins are independently testable — mock events replace real producers
- Plugin registration order matters only for resource initialization, not for event wiring
- Adding a new consumer for an existing event requires no changes to the producer
- Observer events are immediate (same frame), which means ordering within a frame depends on
  observer registration order — this has not caused issues but is worth monitoring
- The boundary check (`mise check:boundary`) enforces the import constraint at CI time
