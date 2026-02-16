---
name: hex-ship
description:
    Run the ship gate audit and verify a cycle is ready to close. Use when a cycle's work is
    complete and ready to ship, or when running the constitution audit before tagging a release.
    Also use when the user invokes /hex-ship.
---

# Ship

Run the constitution audit that gates every release.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name                 | Value                                   | Description                                    |
| -------------------- | --------------------------------------- | ---------------------------------------------- |
| `project_root`       | repository root                         | Base directory; all paths are relative to this |
| `claude_md`          | `{{ project_root }}/CLAUDE.md`          | Ship Gate checks (automated and manual)        |
| `git_guide`          | `{{ project_root }}/docs/guides/git.md` | Ship Merge / Solo-Pitch Merge steps            |
| `contracts_spec_dir` | `{{ project_root }}/docs/contracts`     | Contract specs for parity check                |
| `contracts_src_dir`  | `{{ project_root }}/src/contracts`      | Contract implementations                       |
| `src_dir`            | `{{ project_root }}/src`                | Source directory for unsafe/debug checks       |

## Build Reflection Check

Before running any checks, verify that every pitch in the cycle has a **build reflection comment**
on its issue. This is step 17 in CLAUDE.md's Finishing section. For each pitch:

```bash
gh issue view <pitch-number> --comments
```

Look for a final comment from the build agent that covers:

- What was the final shape vs. the original pitch?
- What was harder or easier than expected?
- What would you do differently if building this scope again?
- What did you learn that future agents (or future cycles) should know?

If any pitch is missing its build reflection, the build agent must post one before the gate
proceeds. Progress updates and completion summaries do not count — the reflection must address the
four questions above.

## Automated Checks

Read `{{ claude_md }}` to extract the Ship Gate section — specifically the automated checks and what
`mise check:audit` covers. Then run the full audit:

```bash
mise check:audit
```

If any check fails, fix the issue and re-run before proceeding.

## Manual Checks

Read `{{ claude_md }}` to extract the manual checks from the Ship Gate section. Walk through each
one with the user. These require human judgment and cannot be automated.

Use `{{ contracts_spec_dir }}` and `{{ contracts_src_dir }}` for the contracts parity check, and
`{{ src_dir }}` for the unsafe and debug checks.

## User Acceptance Testing

After automated and manual checks pass, walk through UAT with the user. For each pitch in the cycle:

1. Read the pitch's plugin spec (`docs/plugins/<name>/spec.md`) and extract the UAT Checklist
   section
2. Present the UAT items to the user
3. The user launches the application, performs each check, and reports pass/fail
4. Record UAT results as a comment on the pitch issue

If any UAT item fails, fix the issue and re-test before proceeding to the gate decision.

## Gate Decision

Present the results to the user:

- **All pass** → proceed with the cycle ship merge.
- **Any fail** → circuit breaker fires. Work does not ship. The problem must be re-shaped and
  re-pitched.

## After the Gate Passes

Read `{{ git_guide }}` to determine the merge workflow:

- **Multi-pitch cycle** (integration branch exists) → follow the Ship Merge steps to merge the
  integration branch to `main`, tag the release, and update `{{ coordination }}`.
- **Solo-pitch cycle** (no integration branch) → follow the Solo-Pitch Merge steps to merge the
  feature branch directly to `main`.
