---
name: hex-observe
description:
    Capture a developer observation as a tagged comment on a GitHub issue during a build cycle. Use
    when the developer notices something worth recording for the retrospective — process friction,
    quality concerns, scope drift, or nascent ideas. Also use when the user invokes /hex-observe.
---

# Observe

Capture a developer observation as a tagged GitHub issue comment so `/hex-retro` can surface it
during the retrospective.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name           | Value                                       | Description                                                 |
| -------------- | ------------------------------------------- | ----------------------------------------------------------- |
| `project_root` | repository root                             | Base directory; all paths are relative to this              |
| `template_dir` | `{{ project_root }}/.github/ISSUE_TEMPLATE` | Issue templates with type labels                            |
| `pitch_label`  | `type:pitch`                                | Label marking pitch issues (read from templates to confirm) |
| `idea_skill`   | `/hex-idea`                                 | Skill to redirect to when an observation is idea-shaped     |
| `obs_marker`   | `[DEV-OBS`                                  | Machine-scannable prefix for developer observation comments |
| `retro_skill`  | `/hex-retro`                                | Retrospective skill that consumes observations              |

## 1. Gather Cycle Context

Identify the active milestone and its pitch issues:

```bash
gh api repos/:owner/:repo/milestones --jq '.[] | select(.state=="open") | {number, title}'
```

If no open milestone exists, warn: "No active cycle found." Ask the user for an explicit issue
number and skip to step 4.

If multiple open milestones exist, present them and ask which one to scope to.

Fetch pitch issues for the selected milestone:

```bash
gh issue list --milestone "<milestone>" --label "type:pitch" --state all --json number,title,body
```

Hold the milestone title and pitch issue list (number, title, body snippet) in memory.

## 2. Hear the Observation

Ask the user: "What did you observe?"

Accept free-text input. A sentence or two is typical; longer is fine.

## 3. Classify and Route

### Infer category

Based on the observation text, assign one of these categories:

- `process` — workflow friction, coordination issues, skill gaps, agent behavior
- `quality` — code quality, architectural drift, testing gaps, convention violations
- `scope` — scope creep, rabbit holes, shape vs. reality divergence, requirements
- `idea` — future work, research needs, tech debt

### Check for idea escalation

If the observation is idea-shaped — uses imperative language ("we should add...", "need to
build..."), describes a capability that does not exist yet, and does not reference what an agent did
or how the current cycle is going — offer to redirect:

"This reads like a new idea rather than a cycle observation. Route to `{{ idea_skill }}` instead?"

- If yes → invoke `{{ idea_skill }}` with the observation text as context. Stop this workflow.
- If no → continue with category `idea`.

### Infer target issue

Compare the observation text against the pitch issue titles and body snippets gathered in step 1.
Pick the best match by keyword overlap.

- If a pitch issue is a clear match → propose that issue.
- If no pitch matches → default to the cycle tracking issue (the milestone's associated cycle issue,
  if one exists) or the most general pitch.
- If still ambiguous → ask the user which issue to post on.

## 4. Confirm and Post

Present the formatted comment and target:

> Post as **[DEV-OBS:category]** on #N (`<issue title>`)?

Show the full comment that will be posted:

```markdown
**[DEV-OBS:category]** — Developer Observation

<observation text>

---

_Logged via `/hex-observe`_
```

If the user wants to change the category or target, adjust and re-confirm.

Once confirmed, post the comment:

```bash
gh issue comment <number> --body "<formatted comment>"
```

Display the comment URL so the user can verify it.

## Quick Capture Mode

If the user provides a fully formed observation with `/hex-observe` (e.g.,
`/hex-observe the terrain agent keeps cycling on clippy fixes for the same file`), skip step 2. Use
the provided text directly and proceed from step 3.
