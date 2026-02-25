# Async Dialog Wrapper Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan
> task-by-task.

**Goal:** Build an async dialog infrastructure (Scope 1 of Pitch #175) that spawns
`rfd::AsyncFileDialog` on Bevy's `IoTaskPool` and polls it each frame, so the event loop never
freezes during native file dialogs.

**Architecture:** A generic `AsyncDialogTask` resource holds an in-flight
`bevy::tasks::Task<DialogResult>`. A polling system in `Update` checks each frame via
`futures_lite::future::block_on(poll_once(...))`. When the task completes, it removes the resource
and triggers a `DialogCompleted` observer event. A guard prevents opening multiple dialogs
simultaneously.

**Tech Stack:** Bevy 0.18 (`bevy::tasks::IoTaskPool`, `bevy::tasks::Task`), `rfd` 0.15
(`AsyncFileDialog`, `AsyncMessageDialog`), `futures-lite` (transitive from Bevy)

---

### Task 1: Add async dialog types to persistence module

**Files:**

- Create: `src/persistence/async_dialog.rs`
- Modify: `src/persistence/mod.rs:18-22` (add module declaration)

**Step 1: Create the async_dialog module with types**

```rust
// src/persistence/async_dialog.rs

//! Async dialog infrastructure.
//!
//! Wraps `rfd::AsyncFileDialog` and `rfd::AsyncMessageDialog` futures behind
//! a Bevy resource so the event loop stays live while a native dialog is open.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::tasks::Task;

/// What action was waiting on the unsaved-changes confirmation dialog.
#[derive(Debug, Clone)]
pub(crate) enum PendingAction {
    /// User triggered Load — after confirmation, open the file picker.
    Load,
    /// User triggered New Project — after confirmation, reset to defaults.
    NewProject { name: String },
    /// User triggered Close Project — after confirmation, return to launcher.
    CloseProject,
}

/// What kind of dialog is in flight and what to do when it finishes.
#[derive(Debug)]
pub(crate) enum DialogKind {
    /// File save picker (save or save-as).
    SaveFile { save_as: bool },
    /// File open picker.
    OpenFile,
    /// Folder picker (export).
    PickFolder,
    /// Unsaved-changes confirmation, with the action to continue after.
    ConfirmUnsavedChanges { then: PendingAction },
}

/// Unified result from any async dialog.
#[derive(Debug)]
pub(crate) enum DialogResult {
    /// User picked a file path, or `None` if cancelled.
    FilePicked(Option<PathBuf>),
    /// User picked a folder path, or `None` if cancelled.
    FolderPicked(Option<PathBuf>),
    /// User responded to a confirmation dialog.
    Confirmed(ConfirmChoice),
}

/// The user's choice in a Yes/No/Cancel confirmation dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfirmChoice {
    Yes,
    No,
    Cancel,
}

/// Holds the in-flight async dialog task. Only one dialog at a time.
///
/// Inserted as a resource when a dialog is spawned; removed when the
/// polling system detects the task has completed.
#[derive(Resource)]
pub(crate) struct AsyncDialogTask {
    /// What kind of dialog is active.
    pub kind: DialogKind,
    /// The spawned async task handle.
    pub task: Task<DialogResult>,
}

// Manual Debug impl because Task<T> does not implement Debug.
impl std::fmt::Debug for AsyncDialogTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncDialogTask")
            .field("kind", &self.kind)
            .field("task", &"<Task>")
            .finish()
    }
}

/// Fired when an async dialog completes. Observer systems handle the result
/// based on the dialog kind.
#[derive(Event, Debug)]
pub(crate) struct DialogCompleted {
    /// What kind of dialog finished.
    pub kind: DialogKind,
    /// The dialog result.
    pub result: DialogResult,
}
```

**Step 2: Add module declaration to mod.rs**

In `src/persistence/mod.rs`, add after line 18 (`pub(crate) mod storage;`):

```rust
pub(crate) mod async_dialog;
```

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | tail -5` Expected: compiles with no errors (types are defined but unused —
clippy warnings expected, will be resolved when systems are added)

**Step 4: Commit**

```
feat(persistence): add async dialog types

Scope 1.1 — AsyncDialogTask resource, DialogKind, DialogResult,
ConfirmChoice, PendingAction, and DialogCompleted event types.
```

---

### Task 2: Add spawn helper functions

**Files:**

- Modify: `src/persistence/async_dialog.rs` (append spawn functions)

**Step 1: Write spawn helpers**

Append to `src/persistence/async_dialog.rs`:

```rust
use bevy::tasks::IoTaskPool;
use hexorder_contracts::storage::Storage;

/// Spawn an async save-file dialog on the I/O thread pool.
///
/// If `initial_dir` is `Some`, the dialog opens in that directory.
/// `file_name` is the suggested filename.
pub(crate) fn spawn_save_dialog(
    initial_dir: Option<&std::path::Path>,
    file_name: &str,
) -> Task<DialogResult> {
    let file_name = file_name.to_string();
    let initial_dir = initial_dir.map(|p| p.to_path_buf());

    IoTaskPool::get()
        .spawn(async move {
            let mut dialog = rfd::AsyncFileDialog::new()
                .add_filter("Hexorder", &["hexorder"])
                .set_file_name(&file_name);

            if let Some(dir) = initial_dir {
                dialog = dialog.set_directory(&dir);
            }

            let result = dialog.save_file().await;
            DialogResult::FilePicked(result.map(|h| h.path().to_path_buf()))
        })
}

/// Spawn an async open-file dialog on the I/O thread pool.
pub(crate) fn spawn_open_dialog() -> Task<DialogResult> {
    IoTaskPool::get()
        .spawn(async move {
            let dialog = rfd::AsyncFileDialog::new()
                .add_filter("Hexorder", &["hexorder"]);

            let result = dialog.pick_file().await;
            DialogResult::FilePicked(result.map(|h| h.path().to_path_buf()))
        })
}

/// Spawn an async folder-picker dialog on the I/O thread pool.
pub(crate) fn spawn_folder_dialog(title: &str) -> Task<DialogResult> {
    let title = title.to_string();

    IoTaskPool::get()
        .spawn(async move {
            let dialog = rfd::AsyncFileDialog::new().set_title(&title);

            let result = dialog.pick_folder().await;
            DialogResult::FolderPicked(result.map(|h| h.path().to_path_buf()))
        })
}

/// Spawn an async unsaved-changes confirmation dialog on the I/O thread pool.
pub(crate) fn spawn_confirm_dialog() -> Task<DialogResult> {
    IoTaskPool::get()
        .spawn(async move {
            let result = rfd::AsyncMessageDialog::new()
                .set_title("Unsaved Changes")
                .set_description(
                    "You have unsaved changes. Do you want to save before continuing?",
                )
                .set_buttons(rfd::MessageButtons::YesNoCancel)
                .set_level(rfd::MessageLevel::Warning)
                .show()
                .await;

            let choice = match result {
                rfd::MessageDialogResult::Yes => ConfirmChoice::Yes,
                rfd::MessageDialogResult::No => ConfirmChoice::No,
                _ => ConfirmChoice::Cancel,
            };
            DialogResult::Confirmed(choice)
        })
}
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -5` Expected: compiles (spawn functions defined, unused warnings
expected)

**Step 3: Commit**

```
feat(persistence): add async dialog spawn helpers

Scope 1.2 — spawn_save_dialog, spawn_open_dialog, spawn_folder_dialog,
spawn_confirm_dialog using IoTaskPool and rfd::AsyncFileDialog.
```

---

### Task 3: Add the polling system

**Files:**

- Modify: `src/persistence/async_dialog.rs` (append polling system)
- Modify: `src/persistence/mod.rs` (register system + observer)

**Step 1: Write the polling system**

Append to `src/persistence/async_dialog.rs`:

```rust
/// Polls the in-flight async dialog each frame.
///
/// Uses `futures_lite::future::block_on(poll_once(...))` which is zero-cost
/// when the future is not yet ready — it returns `None` immediately without
/// blocking.
///
/// When the task completes, removes the `AsyncDialogTask` resource and
/// triggers a `DialogCompleted` observer event.
pub(crate) fn poll_async_dialog(
    task: Option<ResMut<AsyncDialogTask>>,
    mut commands: Commands,
) {
    let Some(mut task) = task else {
        return;
    };

    // Non-blocking poll: returns Some(result) if done, None if pending.
    let Some(result) = futures_lite::future::block_on(
        futures_lite::future::poll_once(&mut task.task),
    ) else {
        return; // Task still in progress.
    };

    // Take ownership of kind before removing the resource.
    // We need to remove the resource first, then use kind.
    // Since we can't move out of ResMut, we'll use commands to remove
    // and trigger separately.
    commands.remove_resource::<AsyncDialogTask>();

    // We need to get the kind out. Since DialogKind doesn't impl Clone,
    // we'll restructure: store the kind separately or make it removable.
    // Actually, the cleanest approach is to use commands.queue for the
    // trigger since we need World access to remove + read.
    commands.queue(move |world: &mut World| {
        // Resource already removed by the remove_resource command above,
        // but commands are deferred. So we remove it here directly.
        if let Some(task_resource) = world.remove_resource::<AsyncDialogTask>() {
            world.commands().trigger(DialogCompleted {
                kind: task_resource.kind,
                result,
            });
        }
    });
}
```

Wait — there's a subtlety. We already polled the task through the `ResMut`, so the result is
extracted. But `kind` is inside the `ResMut` and we can't move it out. Let me reconsider.

The cleaner approach: use an exclusive system or do everything in a `commands.queue` closure.
Actually, the simplest pattern is:

```rust
pub(crate) fn poll_async_dialog(world: &mut World) {
    // Check if there's an active dialog task.
    let Some(mut task_res) = world.get_resource_mut::<AsyncDialogTask>() else {
        return;
    };

    // Non-blocking poll.
    let Some(result) = futures_lite::future::block_on(
        futures_lite::future::poll_once(&mut task_res.task),
    ) else {
        return;
    };

    // Task completed — remove resource and trigger event.
    let task = world.remove_resource::<AsyncDialogTask>()
        .expect("just verified it exists");

    world.commands().trigger(DialogCompleted {
        kind: task.kind,
        result,
    });
}
```

This is an exclusive system (takes `&mut World`), which is fine — it runs once per frame and is
zero-cost when no dialog is active.

**Step 2: Register the polling system in mod.rs**

In `src/persistence/mod.rs`, add to the `build` method after the existing `add_systems(Update, ...)`
call:

```rust
app.add_systems(Update, async_dialog::poll_async_dialog);
```

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | tail -5` Expected: compiles

**Step 4: Commit**

```
feat(persistence): add async dialog polling system

Scope 1.3 — poll_async_dialog exclusive system polls the Task each frame
via block_on(poll_once(...)). Zero-cost when no dialog is active.
Triggers DialogCompleted observer event on completion.
```

---

### Task 4: Write tests for the async dialog infrastructure

**Files:**

- Modify: `src/persistence/async_dialog.rs` (add test module)

**Step 1: Write unit tests**

The key behaviors to test:

1. `poll_async_dialog` is a no-op when no `AsyncDialogTask` resource exists
2. `poll_async_dialog` removes the resource and triggers `DialogCompleted` when the task is ready
3. A dialog is ignored (not spawned) when `AsyncDialogTask` already exists (guard behavior — tested
   at the call site in Scope 2, but we can test the guard concept here)

Note: We cannot test actual `rfd::AsyncFileDialog` in headless tests (no window server). Instead, we
test the polling system with a pre-resolved task.

Append to `src/persistence/async_dialog.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a minimal test app with the polling system.
    fn test_app() -> App {
        let mut app = App::new();
        // TaskPoolPlugin is needed for IoTaskPool::get() to work.
        app.add_plugins(bevy::app::TaskPoolPlugin::default());
        app.add_plugins(bevy::app::TypeRegistrationPlugin);
        app.add_systems(Update, poll_async_dialog);
        app
    }

    /// Polling system is a no-op when no AsyncDialogTask exists.
    #[test]
    fn poll_noop_when_no_task() {
        let mut app = test_app();
        app.update(); // Should not panic.
    }

    /// Polling system removes resource and triggers event when task is ready.
    #[test]
    fn poll_completes_ready_task() {
        let mut app = test_app();

        // Create an already-completed task using IoTaskPool.
        let task = IoTaskPool::get().spawn(async {
            DialogResult::FilePicked(Some(PathBuf::from("/test/file.hexorder")))
        });

        app.insert_resource(AsyncDialogTask {
            kind: DialogKind::SaveFile { save_as: false },
            task,
        });

        // Track whether DialogCompleted was triggered.
        app.add_observer(
            |trigger: On<DialogCompleted>, mut completed: Local<bool>| {
                assert!(matches!(trigger.event().kind, DialogKind::SaveFile { save_as: false }));
                assert!(matches!(
                    &trigger.event().result,
                    DialogResult::FilePicked(Some(p)) if p == &PathBuf::from("/test/file.hexorder")
                ));
                *completed = true;
            },
        );

        // First update: task may complete.
        app.update();
        // Second update: deferred commands apply.
        app.update();

        assert!(
            app.world().get_resource::<AsyncDialogTask>().is_none(),
            "AsyncDialogTask should be removed after completion"
        );
    }

    /// Polling system leaves resource in place when task is pending.
    #[test]
    fn poll_leaves_pending_task() {
        use std::sync::{Arc, Mutex};

        let mut app = test_app();

        // Create a task that blocks on a channel — stays pending.
        let (sender, receiver) = std::sync::mpsc::channel::<()>();
        let sender = Arc::new(Mutex::new(Some(sender)));

        let task = IoTaskPool::get().spawn(async move {
            // Block until signal received (will never come in this test).
            let _ = receiver.recv();
            DialogResult::FilePicked(None)
        });

        app.insert_resource(AsyncDialogTask {
            kind: DialogKind::OpenFile,
            task,
        });

        app.update();

        assert!(
            app.world().get_resource::<AsyncDialogTask>().is_some(),
            "AsyncDialogTask should remain when task is pending"
        );

        // Clean up: drop sender so the task's receiver unblocks.
        drop(sender);
    }

    /// ConfirmChoice round-trips correctly.
    #[test]
    fn confirm_choice_variants() {
        assert_ne!(ConfirmChoice::Yes, ConfirmChoice::No);
        assert_ne!(ConfirmChoice::Yes, ConfirmChoice::Cancel);
        assert_ne!(ConfirmChoice::No, ConfirmChoice::Cancel);
    }

    /// DialogResult debug formatting works.
    #[test]
    fn dialog_result_debug() {
        let result = DialogResult::FilePicked(Some(PathBuf::from("/test")));
        let debug = format!("{result:?}");
        assert!(debug.contains("FilePicked"));
    }
}
```

**Step 2: Run the tests**

Run: `cargo test --lib persistence::async_dialog 2>&1 | tail -20` Expected: all tests pass

**Step 3: Run full test suite**

Run: `cargo test 2>&1 | tail -5` Expected: all tests pass

**Step 4: Commit**

```
test(persistence): add async dialog polling tests

Scope 1.4 — tests for poll_async_dialog (no-op when idle, completes
ready tasks, leaves pending tasks). Uses pre-resolved IoTaskPool tasks
since rfd dialogs require a window server.
```

---

### Task 5: Run quality checks and verify

**Files:** none (verification only)

**Step 1: Run clippy**

Run: `cargo clippy --all-targets 2>&1 | tail -20` Expected: zero warnings

**Step 2: Run full check suite**

Run: `mise check 2>&1 | tail -30` Expected: all checks pass

**Step 3: Fix any issues found**

If clippy or other checks report issues, fix them and re-run.

**Step 4: Commit fixes if needed**

Only if Step 3 required changes.

---

### Task 6: Update spec and log

**Files:**

- Modify: `docs/plugins/persistence/log.md` (append Scope 1 entry)
- Modify: `docs/plugins/persistence/spec.md` (update Constraints section)

**Step 1: Append log entry**

Add to `docs/plugins/persistence/log.md`:

```markdown
## 2026-02-24 — Cycle 8: Async File Dialogs (#175)

### Scope 1: Async Dialog Wrapper

- Created `src/persistence/async_dialog.rs` with infrastructure types:
    - `AsyncDialogTask` resource — holds in-flight `Task<DialogResult>`
    - `DialogKind` enum — SaveFile, OpenFile, PickFolder, ConfirmUnsavedChanges
    - `DialogResult` enum — FilePicked, FolderPicked, Confirmed
    - `ConfirmChoice` enum — Yes, No, Cancel
    - `PendingAction` enum — Load, NewProject, CloseProject
    - `DialogCompleted` observer event
- Spawn helpers: `spawn_save_dialog`, `spawn_open_dialog`, `spawn_folder_dialog`,
  `spawn_confirm_dialog`
- `poll_async_dialog` exclusive system — polls via `block_on(poll_once(...))`, zero-cost when idle
- Uses `IoTaskPool` (not `AsyncComputeTaskPool`) per pitch guidance
- 5 unit tests added, all passing
```

**Step 2: Update spec Constraints**

Add to the Constraints section of `docs/plugins/persistence/spec.md`:

```markdown
- Async file dialogs via `AsyncDialogTask` resource — only one dialog at a time, polled each frame
```

**Step 3: Commit**

```
docs(persistence): log Scope 1 async dialog wrapper
```
