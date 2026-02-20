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

| Name                 | Value                                         | Description                                    |
| -------------------- | --------------------------------------------- | ---------------------------------------------- |
| `project_root`       | repository root                               | Base directory; all paths are relative to this |
| `claude_md`          | `{{ project_root }}/CLAUDE.md`                | Ship Gate checks (automated and manual)        |
| `git_guide`          | `{{ project_root }}/docs/guides/git.md`       | Ship Merge / Solo-Pitch Merge steps            |
| `agent_ops`          | `{{ project_root }}/docs/guides/agent-ops.md` | Agent roles, guard protocol                    |
| `tracking_label`     | `type:cycle`                                  | Label identifying cycle tracking issues        |
| `contracts_spec_dir` | `{{ project_root }}/docs/contracts`           | Contract specs for parity check                |
| `contracts_src_dir`  | `{{ project_root }}/src/contracts`            | Contract implementations                       |
| `src_dir`            | `{{ project_root }}/src`                      | Source directory for unsafe/debug checks       |

## Locate Cycle Tracking Issue

Find the cycle tracking issue and verify readiness prerequisites:

```bash
gh issue list --label "{{ tracking_label }}" --state open --json number,title
gh issue view <tracking-number>
```

From the tracking issue, verify:

- **Integration Setup complete** — all 5 checklist items checked
- **All pitches merged** — every pitch in the Pitch Status table shows "Merged"
- **Lifecycle through item 6** — each pitch issue has lifecycle items 1–6 checked

If any prerequisite is not met, **stop** and report what is missing. The ship gate cannot proceed
with unmerged pitches or incomplete integration setup. Read `{{ agent_ops }}` Guard Protocol for the
full Ship Readiness prerequisites.

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
  integration branch to `main` and tag the release.
- **Solo-pitch cycle** (no integration branch) → follow the Solo-Pitch Merge steps to merge the
  feature branch directly to `main`.

### Changelog Verification

After the changelog is generated (Ship Merge step 8 or Solo-Pitch Merge step 7), verify the output
before committing. Read `CHANGELOG.md` and check:

1. **No `[Unreleased]` header** — the first `##` entry must show the release version and date (e.g.,
   `## [0.10.0] — 2026-02-19`), not `## [Unreleased]`. If `[Unreleased]` appears, git-cliff did not
   recognize the release tag. Check that the tag exists (`git tag -l`) and matches the `tag_pattern`
   in `cliff.toml`. Fix the pattern or tag, then regenerate.
2. **Version matches** — the version in the changelog header matches the version being shipped (from
   `Cargo.toml` and the tag).
3. **Date is correct** — the date matches the tag's commit date.

If any check fails, fix the issue and regenerate before creating the version commit.

After the merge and tag are verified, update the cycle tracking issue:

1. Check off Ship Readiness items on the tracking issue as each completes:
    - All pitches merged to integration branch
    - `mise check:audit` passes on integration branch
    - Manual ship gate checks pass
    - UAT complete
    - Ship merge to main
    - Release tagged and pushed
2. Close the tracking issue:
    ```bash
    gh issue close <tracking-number> --reason completed --comment "Cycle shipped as v<version>."
    ```
