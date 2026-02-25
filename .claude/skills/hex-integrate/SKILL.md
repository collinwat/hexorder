---
name: hex-integrate
description:
    Merge completed pitch branches into the cycle's integration branch. Use when a pitch is ready
    for integration, when coordinating multi-pitch merges, or when verifying integration branch
    health. Also use when the user invokes /hex-integrate.
---

# Integrate

Merge completed pitch branches into the integration branch using rebase + fast-forward.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name             | Value                                                 | Description                                    |
| ---------------- | ----------------------------------------------------- | ---------------------------------------------- |
| `project_root`   | repository root                                       | Base directory; all paths are relative to this |
| `git_guide`      | `{{ project_root }}/docs/guides/git.md`               | Pitch Merge steps, Conflict Resolution         |
| `agent_ops`      | `{{ project_root }}/docs/guides/agent-ops.md`         | Guard protocol, blocking rules                 |
| `cycle_template` | `{{ project_root }}/.github/ISSUE_TEMPLATE/cycle.yml` | Cycle tracking issue structure                 |
| `tracking_label` | `type:cycle`                                          | Label identifying cycle tracking issues        |
| `pitch_label`    | `type:pitch`                                          | Label identifying shaped pitches               |
| `ship_skill`     | `/hex-ship`                                           | Skill to run the ship gate after all merges    |

## 1. Locate the Cycle

Find the active cycle tracking issue and read its current state:

```bash
gh issue list --label "{{ tracking_label }}" --state open --json number,title
gh issue view <tracking-number>
```

From the tracking issue, extract:

- Integration branch name
- Pitch list and current status
- Which pitches have already been merged

Read `{{ cycle_template }}` for expected structure if the tracking issue format is unclear.

## 2. Assess Pitch Readiness

For each pitch in the cycle:

1. Read the pitch issue — check the Lifecycle section:
    ```bash
    gh issue view <pitch-number>
    ```
2. A pitch is **ready** when lifecycle items 1–6 are checked:
    - [x] Branch created from integration branch
    - [x] Build started — kickoff comment posted
    - [x] All build checklist scopes complete
    - [x] Quality gate passed — `mise check:audit`
    - [x] UAT passed — all criteria verified on pitch branch
    - [x] Build reflection posted
    - [x] Ready for integration — spec criteria met, deferred items captured
3. If a pitch is **not ready**, report which items are unchecked and skip it.
4. Present the readiness assessment before proceeding with any merges.

## 3. Integrate a Pitch

Follow the Pitch Merge steps from `{{ git_guide }}`. Read the full procedure there — this is a
summary:

1. **Verify quality gate claim.** Read the pitch issue comments for an audit result confirming
   `mise check:audit` passed.
2. **Verify spec criteria met.** Read `docs/plugins/<name>/spec.md` success criteria.
3. **Verify deferred items captured.** Check spec and log for deferred items — each must have a
   corresponding GitHub Issue.
4. **Rebase feature branch onto integration branch.** Work from the feature branch worktree — never
   `git checkout` on the main working tree.
    ```bash
    cd .worktrees/<release>-<feature>
    git fetch origin <version>
    git rebase origin/<version>
    ```
    If conflicts arise, resolve commit-by-commit. Follow Conflict Resolution rules from
    `{{ git_guide }}`. After resolving, run `mise check:audit`.
5. **Fast-forward merge into integration branch.** Work from the integration branch worktree.
    ```bash
    cd .worktrees/<version>
    git merge --ff-only <release>-<feature>
    ```
6. **Re-test.** Run `mise check:audit` on the integration branch worktree. All checks must pass.
7. **Push.** Push the updated integration branch: `git push origin <version>`.
8. **Update lifecycle.** Check off lifecycle item 8 ("Merged to integration branch") on the pitch
   issue.
9. **Post status comment** on the tracking issue with the merge result:
    ```bash
    gh issue comment <tracking-number> --body "Pitch #<N> (<title>) merged to \`<version>\` ($(git rev-parse --short HEAD)). Audit passed."
    ```

## 4. Check Integration Branch Health

After merging a pitch, verify the integration branch is healthy:

```bash
mise check:audit
```

If any check fails, diagnose and fix before merging additional pitches.

## 5. Assess Ship Readiness

After all pitches have been merged, check the Ship Readiness section on the tracking issue:

- All pitches merged to integration branch
- `mise check:audit` passes on integration branch
- No open blockers

If the cycle is ready for the ship gate, report that and suggest running `{{ ship_skill }}`. Do NOT
run the ship gate from this skill — that is a separate invocation.
