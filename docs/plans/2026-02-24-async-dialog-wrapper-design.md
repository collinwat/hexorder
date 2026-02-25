# Async Dialog Wrapper Design

**Pitch:** #175 — Async file dialogs **Scope:** 1 of 4 — Async dialog wrapper **Date:** 2026-02-24

## Problem

Blocking `rfd::FileDialog` and `rfd::MessageDialog` calls freeze the Bevy event loop while the
native macOS file picker is open. This causes frozen viewport, lost key events, and stuck modifier
keys.

## Solution

An `AsyncDialogTask` resource that wraps `rfd::AsyncFileDialog` futures, polled each frame without
blocking the event loop.

## Architecture

### Flow

```
Observer receives trigger (e.g., SaveRequestEvent)
  → spawns rfd::AsyncFileDialog on IoTaskPool
  → inserts AsyncDialogTask resource
  → returns immediately (event loop stays live)

poll_async_dialog system (Update schedule)
  → checks if AsyncDialogTask resource exists
  → polls via block_on(poll_once(task))
  → when ready: removes resource, triggers completion event
```

### Types

```rust
/// What kind of dialog is active and what happens when it completes.
#[derive(Debug)]
pub enum DialogKind {
    SaveFile { save_as: bool },
    OpenFile,
    PickFolder,
    ConfirmUnsavedChanges { then: PendingAction },
}

/// What action was waiting on the unsaved-changes dialog.
#[derive(Debug, Clone)]
pub enum PendingAction {
    Load,
    NewProject { name: String },
    CloseProject,
}

/// Holds the in-flight async dialog task. Only one dialog at a time.
#[derive(Resource)]
pub struct AsyncDialogTask {
    pub kind: DialogKind,
    pub task: bevy::tasks::Task<DialogResult>,
}

/// Unified result from any dialog.
#[derive(Debug)]
pub enum DialogResult {
    FilePicked(Option<PathBuf>),
    FolderPicked(Option<PathBuf>),
    Confirmed(ConfirmAction),
}

/// Fired when the async dialog completes.
#[derive(Event, Debug)]
pub struct DialogCompleted {
    pub kind: DialogKind,
    pub result: DialogResult,
}
```

### Key decisions

- **IoTaskPool** for spawning (not AsyncComputeTaskPool — file dialogs are I/O, not CPU)
- **futures_lite::future::block_on(poll_once(...))** for zero-cost polling
- **Single resource guard** — if `AsyncDialogTask` exists, new triggers are ignored
- **Completion via observer event** — `DialogCompleted` decouples polling from action
- **No keyboard reset needed** — event loop stays live, so key events are delivered normally

### What this scope delivers

- `AsyncDialogTask` resource, `DialogKind`, `PendingAction`, `DialogResult` types
- Spawn helper functions for each dialog type
- `poll_async_dialog` system registered in `Update`
- `DialogCompleted` event and observer wiring
- Unit tests for polling/completion flow

### What this scope does NOT do

- Migrate existing dialog callers (Scope 2-3)
- Remove the keyboard workaround (Scope 4)
