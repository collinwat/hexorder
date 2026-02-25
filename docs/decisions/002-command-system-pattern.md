# ADR-002: Command System Pattern

**Status:** accepted\
**Date:** 2026-02-25

## Context

Bevy systems can mutate the world in several ways: via `Commands` (deferred, batched), via `Query`
mutation (immediate, scoped to queried components), via `ResMut` (immediate, scoped to one
resource), or via exclusive `&mut World` access (immediate, full access). The project needed a
consistent pattern for when each approach is appropriate.

## Decision

State mutations follow these rules:

1. **Entity lifecycle** (spawn, despawn, insert/remove components) uses `Commands`. This is Bevy's
   intended pattern — structural changes are batched and applied at sync points.

2. **Resource mutation** uses `ResMut<T>` when the system needs to modify a single resource within
   its normal scheduling. Each resource is owned by one plugin; other plugins read it via `Res<T>`.

3. **Component mutation** uses `Query<&mut T>` for in-place updates to existing components. Combined
   with `Changed<T>` filters for efficient change detection.

4. **Event dispatch** uses `commands.trigger(event)` to fire observer events (see ADR-001).

5. **Exclusive world access** (`world: &mut World`) is permitted only when the operation requires
   polymorphic dispatch or multi-resource atomic mutation that cannot be expressed through normal
   system parameters.

**Justified exclusive access** (as of 0.14.0):

| System / Function     | Plugin      | Justification                                             |
| --------------------- | ----------- | --------------------------------------------------------- |
| `process_undo_redo`   | undo_redo   | Calls `UndoableCommand` trait methods taking `&mut World` |
| `commands.queue(...)` | persistence | Async dialog handlers need deferred world access          |
| `save_to_path`        | persistence | Reads multiple registries atomically for serialization    |
| `load_from_path`      | persistence | Overwrites multiple registries atomically on load         |

**Cross-plugin data rules**: Systems must not directly mutate resources owned by other plugins. The
one exception is `load_from_path` in persistence, which intentionally replaces all registries during
a file load — this is the design intent for project deserialization.

**Compliance audit** (0.14.0): All entity lifecycle operations use `Commands`. All resource
mutations use `ResMut<T>`. No unauthorized cross-plugin resource mutation. Exclusive access is
limited to justified cases in undo_redo and persistence.

## Consequences

- Standard systems are parallelizable by Bevy's scheduler — `Commands` and `ResMut` have clear
  borrowing rules that the scheduler can reason about
- Exclusive systems (`&mut World`) block the entire schedule — use sparingly
- The `commands.queue(|world| ...)` pattern bridges async operations (file dialogs) with deferred
  world access, keeping the system itself non-exclusive
- New plugins should default to `Commands` + `ResMut` + `Query` and only escalate to exclusive
  access with documented justification
