//! Async dialog infrastructure.
//!
//! Wraps `rfd::AsyncFileDialog` and `rfd::AsyncMessageDialog` futures behind
//! a Bevy resource so the event loop stays live while a native dialog is open.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll, Waker};

use bevy::prelude::*;

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
#[derive(Debug, Clone)]
pub(crate) enum DialogKind {
    /// File save picker (save or save-as), with optional continuation action.
    SaveFile { then: Option<PendingAction> },
    /// File open picker.
    OpenFile,
    /// Unsaved-changes confirmation, with the action to continue after.
    ConfirmUnsavedChanges { then: PendingAction },
}

/// Unified result from any async dialog.
#[derive(Debug, Clone)]
pub(crate) enum DialogResult {
    /// User picked a file path, or `None` if cancelled.
    FilePicked(Option<PathBuf>),
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

/// Holds the in-flight async dialog future. Only one dialog at a time.
///
/// Inserted as a resource when a dialog is spawned; removed when the
/// polling system detects the future has completed.
#[derive(Resource)]
pub(crate) struct AsyncDialogTask {
    /// What kind of dialog is active.
    pub kind: DialogKind,
    /// The dialog future, polled each frame on the main thread.
    /// Wrapped in `Mutex` to satisfy Bevy's `Resource: Sync` requirement;
    /// only accessed from exclusive `&mut World` systems, so no contention.
    pub future: Mutex<Pin<Box<dyn Future<Output = DialogResult> + Send>>>,
}

// Manual Debug impl because the boxed future does not implement Debug.
impl std::fmt::Debug for AsyncDialogTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncDialogTask")
            .field("kind", &self.kind)
            .field("future", &"<Future>")
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

/// Type alias for a boxed dialog future polled on the main thread.
pub(crate) type DialogFuture = Pin<Box<dyn Future<Output = DialogResult> + Send>>;

/// Create an async save-file dialog future.
///
/// **Must be called on the main thread** so that rfd's macOS backend can
/// present the dialog via `beginSheetModalForWindow` (non-blocking) instead
/// of falling back to `runModal` (blocking).
///
/// If `initial_dir` is `Some`, the dialog opens in that directory.
/// `file_name` is the suggested filename.
pub(crate) fn spawn_save_dialog(
    initial_dir: Option<&std::path::Path>,
    file_name: &str,
) -> DialogFuture {
    let mut dialog = rfd::AsyncFileDialog::new()
        .add_filter("Hexorder", &["hexorder"])
        .set_file_name(file_name);

    if let Some(dir) = initial_dir {
        dialog = dialog.set_directory(dir);
    }

    // Calling save_file() here (on the main thread) creates rfd's
    // ModalFuture which presents the dialog immediately via AppKit.
    let future = dialog.save_file();
    Box::pin(async move {
        let result = future.await;
        DialogResult::FilePicked(result.map(|h| h.path().to_path_buf()))
    })
}

/// Create an async open-file dialog future.
///
/// **Must be called on the main thread** — see [`spawn_save_dialog`].
pub(crate) fn spawn_open_dialog() -> DialogFuture {
    let dialog = rfd::AsyncFileDialog::new().add_filter("Hexorder", &["hexorder"]);
    let future = dialog.pick_file();
    Box::pin(async move {
        let result = future.await;
        DialogResult::FilePicked(result.map(|h| h.path().to_path_buf()))
    })
}

/// Create an async unsaved-changes confirmation dialog future.
///
/// **Must be called on the main thread** — see [`spawn_save_dialog`].
pub(crate) fn spawn_confirm_dialog() -> DialogFuture {
    let future = rfd::AsyncMessageDialog::new()
        .set_title("Unsaved Changes")
        .set_description("You have unsaved changes. Do you want to save before continuing?")
        .set_buttons(rfd::MessageButtons::YesNoCancel)
        .set_level(rfd::MessageLevel::Warning)
        .show();
    Box::pin(async move {
        let result = future.await;
        let choice = match result {
            rfd::MessageDialogResult::Yes => ConfirmChoice::Yes,
            rfd::MessageDialogResult::No => ConfirmChoice::No,
            _ => ConfirmChoice::Cancel,
        };
        DialogResult::Confirmed(choice)
    })
}

/// Polls the in-flight async dialog each frame.
///
/// Uses `Future::poll()` with a noop waker — zero-cost when the future is
/// not yet ready. We poll every frame so waker notification is unnecessary.
///
/// When the future completes, removes the `AsyncDialogTask` resource and
/// triggers a `DialogCompleted` observer event.
pub(crate) fn poll_async_dialog(world: &mut World) {
    // Check if there's an active dialog future.
    let result = {
        let Some(task_res) = world.get_resource_mut::<AsyncDialogTask>() else {
            return;
        };

        // Non-blocking poll: returns Ready(result) if done, Pending otherwise.
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        let mut guard = task_res
            .future
            .lock()
            .expect("dialog future mutex not poisoned");
        match guard.as_mut().poll(&mut cx) {
            Poll::Ready(result) => result,
            Poll::Pending => return, // Future still in progress.
        }
    }; // task_res borrow released here.

    // Future completed — remove resource and trigger event.
    let Some(task) = world.remove_resource::<AsyncDialogTask>() else {
        return;
    };

    world.commands().trigger(DialogCompleted {
        kind: task.kind,
        result,
    });
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::*;

    /// Helper: build a minimal test app with the polling system.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, poll_async_dialog);
        app
    }

    /// Polling system is a no-op when no `AsyncDialogTask` exists.
    #[test]
    fn poll_noop_when_no_task() {
        let mut app = test_app();
        app.update(); // Should not panic.
    }

    /// Polling system removes resource and triggers event when future is ready.
    #[test]
    fn poll_completes_ready_task() {
        let mut app = test_app();

        // Create an already-completed future.
        let future = Box::pin(async {
            DialogResult::FilePicked(Some(PathBuf::from("/test/file.hexorder")))
        });

        app.insert_resource(AsyncDialogTask {
            kind: DialogKind::SaveFile { then: None },
            future: Mutex::new(future),
        });

        // Track whether DialogCompleted was triggered.
        let triggered = Arc::new(AtomicBool::new(false));
        let triggered_clone = Arc::clone(&triggered);
        app.add_observer(move |_trigger: On<DialogCompleted>| {
            triggered_clone.store(true, Ordering::SeqCst);
        });

        // First update: polling system runs, future completes, commands deferred.
        app.update();
        // Second update: deferred commands (trigger) apply.
        app.update();

        assert!(
            app.world().get_resource::<AsyncDialogTask>().is_none(),
            "AsyncDialogTask should be removed after completion"
        );
        assert!(
            triggered.load(Ordering::SeqCst),
            "DialogCompleted observer should have fired"
        );
    }

    /// Polling system leaves resource in place when future is pending.
    #[test]
    fn poll_leaves_pending_task() {
        let mut app = test_app();

        // Create a future that stays pending forever.
        let future: DialogFuture = Box::pin(std::future::pending());

        app.insert_resource(AsyncDialogTask {
            kind: DialogKind::OpenFile,
            future: Mutex::new(future),
        });

        app.update();

        assert!(
            app.world().get_resource::<AsyncDialogTask>().is_some(),
            "AsyncDialogTask should remain when future is pending"
        );
    }

    /// `ConfirmChoice` variants are distinct.
    #[test]
    fn confirm_choice_variants() {
        assert_ne!(ConfirmChoice::Yes, ConfirmChoice::No);
        assert_ne!(ConfirmChoice::Yes, ConfirmChoice::Cancel);
        assert_ne!(ConfirmChoice::No, ConfirmChoice::Cancel);
    }

    /// `DialogResult` debug formatting works.
    #[test]
    fn dialog_result_debug() {
        let result = DialogResult::FilePicked(Some(PathBuf::from("/test")));
        let debug = format!("{result:?}");
        assert!(debug.contains("FilePicked"));
    }
}
