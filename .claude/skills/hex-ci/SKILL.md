---
name: hex-ci
description:
    Diagnose and fix GitHub Actions CI failures. Use when a CI run fails, when the user shares a
    GitHub Actions run URL, or when investigating build status. Also use when the user invokes
    /hex-ci.
---

# CI

Diagnose GitHub Actions CI failures, root-cause them, apply fixes, and re-submit builds.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name           | Value                                         | Description                                             |
| -------------- | --------------------------------------------- | ------------------------------------------------------- |
| `project_root` | repository root                               | Base directory; all paths are relative to this          |
| `ci_workflow`  | `{{ project_root }}/.github/workflows/ci.yml` | CI workflow definition — job names, runners, mise tasks |
| `mise_config`  | `{{ project_root }}/mise.toml`                | Mise task definitions referenced by CI jobs             |
| `git_guide`    | `{{ project_root }}/docs/guides/git.md`       | Commit format for fix commits                           |
| `hook_config`  | `{{ project_root }}/lefthook.yml`             | Pre-commit hooks that mirror CI checks locally          |
| `ship_gate`    | `CLAUDE.md` → Ship Gate                       | Canonical list of checks the CI enforces                |

## 1. Identify the Run

Determine which CI run to investigate:

- **URL provided**: Extract the run ID from the GitHub Actions URL.
- **No URL**: Find the latest failed run:

```bash
gh run list --status failure --limit 5
```

If the user names a specific branch or PR, scope the search:

```bash
gh run list --branch <branch> --status failure --limit 5
```

## 2. Get Failure Summary

Fetch the run metadata and identify which jobs failed:

```bash
gh run view <run_id> --json jobs --jq '.jobs[] | select(.conclusion == "failure") | {name: .name, conclusion: .conclusion}'
```

Present the list of failed job names to establish scope.

## 3. Read Failure Logs

For each failed job, fetch the logs and extract the error:

```bash
gh run view <run_id> --log-failed 2>&1
```

If the output is large, filter for error signals:

```bash
gh run view <run_id> --log-failed 2>&1 | grep -E '(^error|FAILED|panicked|##\[error|warning\[|VIOLATION|BOUNDARY)' | head -80
```

Read `{{ ci_workflow }}` to understand the job's runner, steps, and which mise task it runs. This
maps the job name to the local command that reproduces it.

## 4. Classify the Failure

Categorize the root cause into one of these classes:

### Transient / Infrastructure

- Network errors (502, 503, timeout) downloading dependencies or tools
- Runner provisioning failures
- Cache miss or corruption
- GitHub API rate limits

**Action**: Skip to step 7 (re-run). No code fix needed.

### Code Error

- Compilation failure (`cargo build` / `cargo test`)
- Clippy warning (pedantic lint violation)
- Test failure (assertion, panic)
- Documentation build error

**Action**: Proceed to step 5.

### Formatting / Style

- `cargo fmt` diff
- Prettier diff
- TOML formatting (`taplo`)
- Typos

**Action**: Proceed to step 5 — these are auto-fixable.

### Policy Violation

- `unwrap()` in production code
- Cross-plugin import boundary violation
- Missing `Debug` derive
- Secret detected (gitleaks)

**Action**: Proceed to step 5.

## 5. Reproduce Locally

Run the equivalent local command to confirm the failure reproduces. Read `{{ ci_workflow }}` to find
the mise task the failed job runs, then execute it:

```bash
mise run <task>
```

Common mappings (read `{{ ci_workflow }}` for the authoritative list):

| CI Job            | Local Command             |
| ----------------- | ------------------------- |
| Format            | `mise run check:fmt`      |
| Clippy            | `mise run check:clippy`   |
| Test              | `mise run test`           |
| Dependency Audit  | `mise run check:deny`     |
| Spell Check       | `mise run check:typos`    |
| TOML Format       | `mise run check:taplo`    |
| Prettier          | `mise run check:prettier` |
| Import Boundaries | `mise run check:boundary` |
| No Unwrap         | `mise run check:unwrap`   |
| Debug Derive      | `mise run check:debug`    |
| Documentation     | `mise run check:doc`      |

If the failure does NOT reproduce locally, note the discrepancy — it may be platform-specific (the
CI uses both `ubuntu-latest` and `macos-latest`) or environment-dependent.

## 6. Fix

Apply the fix based on the failure class:

### Auto-fixable (formatting, typos, TOML)

```bash
mise run fix
```

Then verify the fix:

```bash
mise run check
```

### Code / Policy errors

Read the relevant source files, understand the violation, and fix it. After fixing, re-run the
specific check that failed:

```bash
mise run <task>
```

Then run the full check suite to ensure no regressions:

```bash
mise run check
```

### Multiple jobs failed

Fix all failures before committing. Run `mise run check` to verify everything passes together.

## 7. Re-submit

### Transient failure (no code change needed)

Re-run only the failed jobs:

```bash
gh run rerun <run_id> --failed
```

### Code fix applied

Commit the fix using the project's commit conventions. Read `{{ git_guide }}` for the commit format.
Use type `fix` with the appropriate scope:

```
fix(<scope>): <description of what was wrong>
```

Push the fix. CI will trigger automatically on the push. Verify the new run starts:

```bash
gh run list --branch <branch> --limit 1
```

## 8. Verify

After re-run or push, monitor the new run:

```bash
gh run watch <new_run_id>
```

If it fails again, loop back to step 3 with the new run ID.

Report the final status to the user: which jobs passed, which (if any) still need attention.

## Quick Mode

When the user provides a run URL and the failure is obviously transient (network error, 502, runner
issue), skip reproduction and go straight to re-run. Report what happened and that the re-run was
submitted.
