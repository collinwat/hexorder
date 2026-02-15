---
name: hex-commit
description:
    Commit staged or unstaged changes with proper atomic commit hygiene. Analyzes changes for
    distinct concerns, proposes splitting when appropriate, generates conventional commit messages
    that pass the project's commit-msg hook, and handles failures. Use when the user wants to commit
    work, or when the user invokes /hex-commit.
---

# Commit

Commit changes with atomic commit discipline — each commit represents one logical change.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name              | Value                                   | Description                                            |
| ----------------- | --------------------------------------- | ------------------------------------------------------ |
| `project_root`    | repository root                         | Base directory; all paths are relative to this         |
| `git_guide`       | `{{ project_root }}/docs/guides/git.md` | Commit format, types, scopes, and pre-commit checklist |
| `hook_config`     | `{{ project_root }}/lefthook.yml`       | Hook commands that validate commits                    |
| `changelog_guide` | `{{ git_guide }}` → Changelog Format    | How commit types map to changelog sections             |

## 1. Learn the Commit Standards

Read `{{ git_guide }}` to extract the project's current commit conventions. Specifically, find and
hold in memory:

- **Commit message format** — the required structure (from the "Commit message format" section)
- **Valid types** — the type table (e.g., `feat`, `fix`, `refactor`, etc.) with descriptions
- **Valid scopes** — the scope table (e.g., plugin names, `contracts`, `project`) with descriptions
- **Subject line rules** — imperative mood, casing, punctuation, length limits
- **Body rules** — when to include a body, wrap width, content guidance
- **Pre-Commit Checklist** — the checklist steps that apply to every commit

Also read `{{ hook_config }}` to understand what the `commit-msg` hook validates, so generated
messages will pass on the first attempt.

Do NOT hardcode types, scopes, or format rules — always read them fresh from the files.

## 2. Check for Changes

Run `git status`. If there are no staged or unstaged changes, say so and stop.

## 3. Show Changes

Display what's available to commit:

- **Staged changes**: `git diff --cached --stat` (and `git diff --cached` for content)
- **Unstaged changes**: `git diff --stat` (and `git diff` for content)
- **Untracked files**: from the `git status` output

Present a clear summary so the user can see the full picture.

## 4. Select Scope

Ask the user what to commit. Offer options based on what exists:

- **Staged only** — commit exactly what's staged (skip to step 6)
- **All changes** — stage and commit everything (proceed to step 5)
- **Specific files** — let the user pick which files to include (proceed to step 5)

## 5. Analyze for Atomic Commits

If committing unstaged or all changes that span multiple unrelated concerns, identify distinct
changesets that should be separate commits. Look for:

- **Different types** — a bug fix mixed with a new feature
- **Different scopes** — changes to one plugin mixed with changes to another
- **Different purposes** — a refactor mixed with a documentation update
- **Unrelated file groups** — files that serve independent goals

If the changes are naturally atomic (single concern, single scope), proceed to step 6.

If splitting is warranted, propose the split to the user. Show which files belong to each proposed
commit and what the commit message would be. Let the user confirm or override.

Skip this step if the user selected "staged only" — trust that they staged intentionally.

## 6. Generate Commit Message

Using the types, scopes, and format rules learned in step 1, draft a commit message:

- Match the type to the nature of the change
- Match the scope to the affected area
- Write the subject in imperative mood, describing the _why_ not the _what_
- Add a body if the change warrants explanation (wrap at the width specified in the guide)
- Include issue references if the changes address tracked items

Verify the message would pass the `commit-msg` hook validation from `{{ hook_config }}` before
presenting it.

## 7. Confirm

Display the full commit message in plain text (not a code block). Then ask: "Ready to commit, or
would you like to change anything?"

Never commit without explicit confirmation.

## 8. Commit

Stage files as needed and create the commit. Verify with `git log -1 --oneline`.

## 9. Handle Failures

- **Pre-commit hook fails**: Show the error output. Ask permission before attempting to fix. If
  allowed, fix the issue and loop back to step 2.
- **Commit-msg hook fails**: The message didn't match the format. Re-read `{{ hook_config }}` to
  understand the rejection, regenerate the message, and loop back to step 7.
- **Other errors**: Show the error and stop. Let the user decide next steps.

## 10. Repeat if Splitting

If changes were split into multiple commits in step 5, loop back to step 4 for the remaining
changes. Continue until all proposed commits are made or the user stops.
