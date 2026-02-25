# Hexorder — Agent Operations Guide

This guide defines operational patterns shared by both agent roles (pitch agent and cycle agent).
Agents reference this guide for resume, guard, sync, and blocking protocols.

For agent definitions, see `.claude/agents/pitch-agent.md` and `.claude/agents/cycle-agent.md`. For
git procedures, see `docs/guides/git.md`.

---

## Agent Roles

Hexorder uses two agent roles during the build phase of a Shape Up cycle. Each role has clear
ownership boundaries and coordinates through GitHub Issue checklists.

### Pitch Agent

- **Scope**: one pitch, one feature branch, one worktree
- **Owns**: feature branch + worktree
- **Skills**: `/hex-kickoff`, `/hex-commit`, `/hex-idea`, `/hex-continue`, `/hex-status`
- **Lifecycle items**: 1–6 on the pitch issue's Lifecycle section
- **Handoff**: checks "Ready for integration" when its work is done

### Cycle Agent

- **Scope**: the entire build cycle, the integration branch
- **Owns**: integration branch + cycle tracking issue
- **Skills**: `/hex-integrate`, `/hex-ship`, `/hex-continue`, `/hex-status`
- **Lifecycle items**: item 7 (Merged to integration branch) on each pitch issue
- **Tracking**: manages the cycle tracking issue's Integration Setup, Pitch Status, and Ship
  Readiness sections

### Handoff

The handoff point is the pitch Lifecycle checklist item "Ready for integration." Once the pitch
agent checks it, the cycle agent can proceed with the integration. The checklist IS the coordination
mechanism — no direct communication between agents is needed.

---

## Continue/Resume Protocol

When an agent session starts (or restarts after interruption), follow this protocol to determine the
current state and next action. The `/hex-continue` skill implements this protocol.

1. **Read the relevant checklist.**
    - Pitch agent → pitch issue Lifecycle section
    - Cycle agent → cycle tracking issue (Integration Setup, Pitch Status, Ship Readiness)

2. **Find the last checked item.** That's the last completed phase. The next unchecked item is the
   next action.

3. **Read git state.**
    - `git branch --show-current` — which branch?
    - `git status --short` — uncommitted changes?
    - `git log --oneline -5` — recent commits?
    - For pitch agents: `git rev-list --count HEAD..origin/<version>` — is the integration branch
      ahead?

4. **Read issue tracker state.**
    - Recent comments on the pitch issue or tracking issue
    - Labels and assignees
    - Open blockers (`gh issue list --label "status:blocked" --state open`)

5. **Determine next action** based on the gap between checklist state and git state. The checklist
   shows what's been completed; the git state shows the current working state. If there's a gap
   (e.g., checklist says "Build started" but there are no commits), investigate.

6. **Handle uncommitted work.** If uncommitted changes exist from a prior session:
    - Review the changes: `git diff`
    - If coherent and useful: commit with
      `chore(<feature>): recover uncommitted work from prior session`
    - If broken or incomplete: discard with `git checkout -- .`

7. **Report.** Present to the user (or agent orchestrator):
    - What happened last (last checked lifecycle item + recent commits)
    - What comes next (next unchecked item + specific action)
    - Any blockers (failed checks, missing prerequisites, sync needed)

References `docs/guides/git.md` Session Handoff Protocol for additional git-level details.

---

## Guard Protocol

Each lifecycle item has prerequisites. Before checking off item N, verify that items 1 through N-1
are checked and the item's specific conditions are met. This prevents premature advancement and
ensures restart recovery — a new agent can trust that checked items are truly complete.

### Pitch Lifecycle Guards

| Item                  | Prerequisites                                                                                                                                                |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Branch created        | Integration branch exists (remotely or locally), feature branch created from it, worktree set up                                                             |
| Build started         | Branch exists, kickoff comment posted on pitch issue, tracking issue updated                                                                                 |
| Scopes complete       | Every item in the Build Checklist section is checked off                                                                                                     |
| Gate passed           | `mise check:audit` passes on the feature branch with zero failures                                                                                           |
| Reflection posted     | Final reflection comment on pitch issue addresses: final shape vs. original pitch, harder/easier than expected, what to do differently, learnings for future |
| Ready for integration | Gate passed + reflection posted + every spec success criterion met + all deferred items have GitHub Issues                                                   |
| Merged to integration | Cycle agent performed rebase + fast-forward merge + `mise check:audit` passes on integration branch                                                          |

### Cycle Tracking Guards

| Section                    | Prerequisites                                                                                                                             |
| -------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| Integration Setup complete | All 5 checklist items checked, branch exists on remote, milestone description updated                                                     |
| Pitch merged               | Pitch lifecycle item 6 ("Ready for integration") is checked, rebase + FF merge succeeded, `mise check:audit` passes on integration branch |
| Ship readiness             | All pitches show "Merged" status, `mise check:audit` passes, manual ship gate checks pass, UAT complete                                   |

---

## Sync Protocol

Feature branches fall behind as other pitches are integrated into the integration branch. Pitch
agents must periodically sync to stay current and minimize conflicts during integration.

### When to sync

- Before starting a new scope (to build on the latest integrated state)
- After another pitch is integrated (the tracking issue comment thread announces integrations)
- Before declaring "Ready for integration" (to ensure a clean rebase during integration)

### How to sync

See the Feature Branch Sync section in `docs/guides/git.md`. In brief:

```bash
git fetch origin <version>
git rebase origin/<version>
git push --force-with-lease origin <release>-<feature>
```

Force-push with `--force-with-lease` is safe because each feature branch is owned by a single pitch
agent.

### Cycle agent sync

The cycle agent does not need to sync the integration branch during the build phase — it receives
fast-forward merges from rebased pitch branches. At ship time, the integration branch rebases onto
`main` per the Ship Merge steps in `docs/guides/git.md`.

---

## Reflection Protocol

Reflection checkpoints force agents to pause and evaluate their approach. The value is in the pause,
not in lengthy essays — keep reflections to 2-3 sentences.

### After each scope

Post a comment on the pitch issue after completing each build checklist scope. Answer these prompts:

- What assumption did I just test?
- Is there a simpler approach I missed?
- What would I tell the next agent about this scope?

Include lines changed (e.g., "+120/-30 across 4 files") for visibility.

### After a blocker or debugging session > 30 minutes

Before continuing, post a reflection comment on the pitch issue. Cover:

- What was the symptom and what did I try?
- What was the root cause?
- How could this have been caught earlier?

This prevents sunk-cost momentum — the pause itself is the intervention.

### After agent handoff

When a new agent session picks up an existing feature branch, post an orientation comment on the
pitch issue summarizing:

- What was found (last completed phase, uncommitted work, open questions)
- What comes next (next unchecked scope, blockers)
- Any discrepancies between the checklist state and the actual branch state

This creates a visible handoff record in the pitch issue comment thread.

---

## Blocking Rules

These rules prevent premature or unauthorized actions. The guard protocol enforces them — the
checklist IS the coordination mechanism.

- A **pitch agent** must NOT merge its own branch to the integration branch. That is the cycle
  agent's responsibility. The pitch agent's job ends at "Ready for integration."
- A **cycle agent** must NOT merge a pitch that hasn't checked "Ready for integration" on its
  Lifecycle. Always verify lifecycle item 6 before proceeding.
- **Neither agent** runs the ship gate without all pitches merged to the integration branch.
- A **pitch agent** must NOT run the ship gate. Only the cycle agent (or the orchestrator via
  `/hex-ship`) does this.
- A **cycle agent** must NOT modify pitch feature branches. It only operates on the integration
  branch by rebasing and fast-forward merging pitch branches.

---

## Task List Coordination

Multi-terminal cycles use `CLAUDE_CODE_TASK_LIST_ID` to share live task visibility across Claude
Code sessions. Each session can see and update the same task list, providing task-level granularity
beyond what GitHub Issue checklists offer.

### Setup

Set the environment variable before launching Claude Code sessions for a cycle:

```bash
export CLAUDE_CODE_TASK_LIST_ID=hexorder-<version>
```

For example, `hexorder-0.11.0` for the 0.11.0 cycle. All sessions in the cycle use the same ID.

### Naming conventions

Tasks should follow this pattern:

```
[<pitch-name>] <scope description>
```

Examples:

- `[dockable-panels] Evaluate egui_dock integration`
- `[build-discipline] Add cargo safety guardrails`
- `[cycle-agent] Integrate pitch #134`

### When to use

- **Pitch agents**: Create tasks for each scope in the Build Checklist. Update status as work
  progresses. This gives the cycle agent and other pitch agents live visibility into progress.
- **Cycle agent**: Create tasks for integration and ship gate steps. Visible to all pitch agents so
  they can see when integrations are happening.
- **Orchestrator**: Review the shared task list for full cycle progress at a glance.

### Relationship to GitHub Issues

The task list supplements, not replaces, GitHub Issue checklists. Issue Lifecycle items remain the
official coordination mechanism for agent handoffs. The task list provides finer-grained, real-time
visibility within a session.
