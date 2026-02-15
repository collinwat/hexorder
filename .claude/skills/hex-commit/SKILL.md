---
name: hex-commit
description:
    Commit staged or unstaged changes with proper atomic commit hygiene. Use when the user wants to
    commit work. Also use when the user invokes /hex-commit.
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

## 4. Determine What to Commit

Decide what to include based on the current state:

- **Staged changes exist** — commit exactly what's staged. Trust the user's staging.
- **No staged changes** — include all unstaged and untracked changes.

If including unstaged/untracked changes that span multiple unrelated concerns, analyze for
splitting. Look for:

- **Different types** — a bug fix mixed with a new feature
- **Different scopes** — changes to one plugin mixed with changes to another
- **Different purposes** — a refactor mixed with a documentation update
- **Unrelated file groups** — files that serve independent goals

If the changes are naturally atomic (single concern, single scope), proceed to step 5.

If splitting is warranted, propose the split to the user. Show which files belong to each proposed
commit and what the commit message would be. Let the user confirm or override.

## 5. Generate Commit Message

Using the types, scopes, and format rules learned in step 1, draft a commit message:

- Match the type to the nature of the change
- Match the scope to the affected area
- Write the subject in imperative mood, describing the _why_ not the _what_
- Add a body if the change warrants explanation (wrap at the width specified in the guide)
- Include issue references if the changes address tracked items

Verify the message would pass the `commit-msg` hook validation from `{{ hook_config }}` before
presenting it.

## 6. Commit

Present the proposed commit message, then stage files as needed and create the commit in a single
step. The user's approval of the git command serves as confirmation — do not ask separately.

Verify with `git log -1 --oneline`.

## 7. Handle Failures

- **Pre-commit hook fails**: Show the error output. Ask permission before attempting to fix. If
  allowed, fix the issue and loop back to step 2.
- **Commit-msg hook fails**: The message didn't match the format. Re-read `{{ hook_config }}` to
  understand the rejection, regenerate the message, and loop back to step 5.
- **Other errors**: Show the error and stop. Let the user decide next steps.

## 8. Repeat if Splitting

If changes were split into multiple commits in step 4, loop back to step 4 for the remaining
changes. Continue until all proposed commits are made or the user stops.
