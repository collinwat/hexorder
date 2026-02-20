---
name: hex-bisect
description:
    Triage a crash or regression by systematically eliminating common causes before resorting to
    source code bisection. Use when an app crash, test failure, or runtime regression appears with
    no obvious cause. Also use when the user invokes /hex-bisect.
---

# Bisect

Structured crash triage — eliminate environmental causes before bisecting source code.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name           | Value                                   | Description                                            |
| -------------- | --------------------------------------- | ------------------------------------------------------ |
| `project_root` | repository root                         | Base directory; all paths are relative to this         |
| `git_guide`    | `{{ project_root }}/docs/guides/git.md` | Branching model, worktree setup, shared target dir doc |
| `cargo_toml`   | `{{ project_root }}/Cargo.toml`         | Dependency declarations and feature flags              |
| `cargo_lock`   | `{{ project_root }}/Cargo.lock`         | Pinned dependency versions                             |
| `hook_config`  | `{{ project_root }}/lefthook.yml`       | Hook commands including cargo-lock-guard               |
| `mise_config`  | `{{ project_root }}/mise.toml`          | Task definitions for check, test, build                |

## 1. Reproduce and Record

Before investigating, confirm the failure is reproducible:

1. Run the exact command that triggered the crash or regression.
2. Record the error output, signal (e.g., SIGABRT, SIGSEGV), and any backtrace.
3. Note whether the failure is deterministic or intermittent.

If the failure is not reproducible, record the original observation and stop — intermittent failures
need a different approach (logging, stress testing).

## 2. Check Cargo.lock and Cargo.toml Drift

Dependency drift is the most common invisible cause of runtime crashes on feature branches.

```bash
git diff origin/<parent-branch> -- Cargo.lock Cargo.toml
```

Replace `<parent-branch>` with the integration branch (e.g., `0.11.0`) or `main` as appropriate.

- **If `Cargo.lock` has changed** and the changes are not from an intentional `Cargo.toml` edit:
    1. Revert `Cargo.lock` to the parent branch version:
        ```bash
        git checkout origin/<parent-branch> -- Cargo.lock
        ```
    2. Rebuild and retest:
        ```bash
        cargo build --features dev
        cargo test
        ```
    3. If the crash disappears, the dependency bump was the cause. Commit the reverted `Cargo.lock`
       and document which transitive dependency caused the issue.
    4. If the crash persists, restore the current `Cargo.lock` and proceed to step 3.

- **If `Cargo.toml` has changed** (new dependencies added):
    1. Check whether the new dependency versions are compatible:
        ```bash
        cargo tree -i <suspect-crate> --depth 1
        ```
    2. If a newly added dependency pulls in a conflicting version, pin the correct version in
       `Cargo.toml` and rebuild.

- **If neither has changed**, proceed to step 3.

## 3. Check Feature Flags

Feature flags can activate code paths that crash on specific platforms.

```bash
cargo build --features dev
cargo build --no-default-features
cargo build
```

Read `{{ cargo_toml }}` to find the `[features]` section. Test with each feature individually:

```bash
cargo build --features <flag>
```

If the crash is feature-dependent, narrow down which feature flag activates the crashing code path.
Report which flag triggers the failure before proceeding.

## 4. Clean Stale Artifacts

Git worktrees share a single `target/` directory by default. Stale build artifacts from one worktree
can contaminate builds in another.

```bash
cargo clean -p hexorder
cargo build --features dev
```

If the crash disappears after cleaning, the cause was stale artifacts from the shared target
directory. Document this in the pitch issue comment.

> **Shared target directory risk**: All worktrees under the same repository share
> `<repo-root>/target/`. When one worktree builds with different features or dependency versions,
> incremental compilation artifacts may become invalid for other worktrees.
> `cargo clean -p hexorder` removes only the project's own artifacts, forcing a rebuild without
> wiping the entire dependency cache.

## 5. Source Code Bisection

Only reach this step after ruling out dependency drift, feature flags, and stale artifacts.

1. Identify the known-good commit (last commit where the behavior was correct):
    ```bash
    git log --oneline -20
    ```
2. Use git bisect to find the offending commit:
    ```bash
    git bisect start
    git bisect bad HEAD
    git bisect good <known-good-sha>
    ```
3. At each bisect step, build and test:
    ```bash
    cargo build --features dev && cargo test
    ```
    Then mark:
    ```bash
    git bisect good   # if no crash
    git bisect bad    # if crash reproduces
    ```
4. When bisect identifies the first bad commit, examine it:
    ```bash
    git bisect reset
    git show <bad-commit>
    ```
5. Report the offending commit and the root cause.

## 6. Document

Post a comment on the pitch issue summarizing:

- The symptom (crash signal, error message)
- Which triage step identified the cause (dependency drift, feature flag, stale artifacts, or source
  bisection)
- The fix applied
- Any preventive measure added (e.g., Cargo.lock reverted, clean step added to workflow)
