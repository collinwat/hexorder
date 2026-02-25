# Async Dialog Migration Design (Scope 2)

> **Goal:** Migrate all blocking `rfd::FileDialog` and `rfd::MessageDialog` calls in
> `src/persistence/systems.rs` to the async infrastructure from Scope 1.

## Current State

Four blocking dialog call sites in `systems.rs`:

| Call site                           | Dialog type                    | Callers                          |
| ----------------------------------- | ------------------------------ | -------------------------------- |
| `do_save()` L164                    | `rfd::FileDialog::save_file()` | save, load, new, close observers |
| `check_unsaved_changes()` L122      | `rfd::MessageDialog::show()`   | load, new, close observers       |
| `handle_load_request()` L358        | `rfd::FileDialog::pick_file()` | direct                           |
| `clear_keyboard_after_dialog()` L40 | workaround for frozen events   | all dialog sites                 |

The main complexity is **dialog chaining**: `check_unsaved_changes` may trigger `do_save`, which may
show its own dialog. This creates a two-deep blocking chain.

## Design

### Type Change: Add `then` to `SaveFile`

Extend `DialogKind::SaveFile` with an optional continuation action:

```rust
SaveFile { save_as: bool, then: Option<PendingAction> }
```

When a save dialog completes and `then` is `Some`, execute the pending action after saving. This
enables the confirm → save → action chain without nested blocking.

### Extract Pure Logic Helpers

Split the current `do_save()` into dialog-free helpers:

- `save_to_path(path, workspace, ..., storage, commands) -> bool` — builds `GameSystemFile`, writes
  to disk, updates workspace. No dialog logic. Returns success/failure.
- `load_from_path(path, ..., commands)` — reads file, overwrites registries, inserts
  `PendingBoardLoad`. No dialog logic.
- `reset_to_new_project(name, ..., commands)` — resets all registries to defaults, sets workspace
  name, transitions to Editor.
- `close_project_reset(..., commands)` — resets all state, transitions to Launcher.

These helpers are called from both the existing trigger observers (synchronous fast path) and the
new `handle_dialog_completed` observer (async completion path).

### Phase 1: Trigger Observers Spawn Async Dialogs

Each existing observer becomes a thin dispatcher:

**`handle_save_request`**:

1. Guard: if `AsyncDialogTask` exists, return (dialog already open).
2. If `save_as` or no existing path → spawn async save dialog, insert `AsyncDialogTask`.
3. If existing path → call `save_to_path()` directly (no dialog needed).

**`handle_load_request`**:

1. Guard: if `AsyncDialogTask` exists, return.
2. If `workspace.dirty` → spawn async confirm dialog with `PendingAction::Load`.
3. If not dirty → spawn async open dialog directly.

**`handle_new_project`**:

1. Guard: if `AsyncDialogTask` exists, return.
2. If `workspace.dirty` → spawn async confirm dialog with `PendingAction::NewProject { name }`.
3. If not dirty → call `reset_to_new_project()` directly.

**`handle_close_project`**:

1. Guard: if `AsyncDialogTask` exists, return.
2. If `workspace.dirty` → spawn async confirm dialog with `PendingAction::CloseProject`.
3. If not dirty → call `close_project_reset()` directly.

### Phase 2: Dialog Completion Handler

A new `handle_dialog_completed` observer processes `DialogCompleted` events. This is the central
result router:

**`ConfirmUnsavedChanges { then }` + `Confirmed(choice)`**:

- `Yes` → save first, then do `then`:
    - If workspace has path → `save_to_path()`, then execute `then`
    - If no path → spawn save dialog with `then: Some(then)` (chains to save completion)
- `No` → execute `then` directly (skip save)
- `Cancel` → do nothing

**`SaveFile { save_as, then }` + `FilePicked(Some(path))`**:

- Call `save_to_path(path, ...)`
- If `then` is `Some` → execute the pending action

**`SaveFile { .. }` + `FilePicked(None)`**:

- User cancelled — do nothing (abort the chain)

**`OpenFile` + `FilePicked(Some(path))`**:

- Call `load_from_path(path, ...)`

**`OpenFile` + `FilePicked(None)`**:

- User cancelled — do nothing

### Execute Pending Action

A helper function dispatches `PendingAction` variants:

```rust
fn execute_pending_action(action: PendingAction, world: &mut World) {
    match action {
        PendingAction::Load => { /* spawn open dialog */ }
        PendingAction::NewProject { name } => { /* reset_to_new_project */ }
        PendingAction::CloseProject => { /* close_project_reset */ }
    }
}
```

`PendingAction::Load` spawns another async dialog (the open-file picker). The other variants execute
immediately since they don't need dialogs.

### Observer Access Pattern

`handle_dialog_completed` needs broad access to registries. Since it's an exclusive system observer
(takes `&mut World`), it can access everything directly. The pure logic helpers accept individual
references/queries to keep their signatures testable.

### What Gets Removed

- `check_unsaved_changes()` — replaced by async confirm dialog
- `ConfirmAction` enum — replaced by `ConfirmChoice` from async_dialog
- `clear_keyboard_after_dialog()` — no longer needed (event loop never freezes)
- All `rfd::FileDialog` and `rfd::MessageDialog` imports from systems.rs
- All `clear_keyboard_after_dialog` calls

## Test Strategy

- Existing tests for `apply_pending_board_load`, `sync_dirty_flag`, `sync_window_title` are
  unchanged (these systems don't involve dialogs).
- New tests for `save_to_path` and `load_from_path` using the existing `Storage` mock/provider.
- The `handle_dialog_completed` state machine is tested by inserting pre-resolved `AsyncDialogTask`
  resources and verifying the correct action executes.
- Existing `poll_async_dialog` tests from Scope 1 validate the polling infrastructure.
