---
name: hex-retro
description:
    Run a cycle retrospective to reflect on what shipped, what was learned, and capture new ideas as
    GitHub Issues. Use at the end of a build cycle during cool-down, before shaping and betting for
    the next cycle. Also use when the user invokes /hex-retro.
---

# Retro

Run a structured retrospective at the end of a build cycle. The goal is reflection and idea capture
— not commitments. New ideas become GitHub Issues for potential future shaping.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name           | Value                                       | Description                                    |
| -------------- | ------------------------------------------- | ---------------------------------------------- |
| `project_root` | repository root                             | Base directory; all paths are relative to this |
| `coordination` | `{{ project_root }}/docs/coordination.md`   | Active cycle, bets, milestones                 |
| `plugins_dir`  | `{{ project_root }}/docs/plugins`           | Plugin spec and log directory                  |
| `template_dir` | `{{ project_root }}/.github/ISSUE_TEMPLATE` | Issue templates with type labels               |
| `wiki_dir`     | `.wiki`                                     | GitHub Wiki local clone                        |
| `wiki_home`    | `{{ wiki_dir }}/Home.md`                    | Wiki landing page                              |

## Gather Context

1. Read `{{ coordination }}` to extract the current cycle's bets and milestones.
2. Review the cycle's git history:
    ```bash
    git log --oneline --since="<cycle-start>" --until="<cycle-end>"
    ```
3. Read plugin logs for plugins worked on this cycle: `{{ plugins_dir }}/<name>/log.md`
4. **Read the build agent's voice** — fetch comments from the cycle's pitch issues. These contain
   the agent's progress updates and build reflection posted during the build phase:
    ```bash
    gh issue view <pitch-number> --comments
    ```
    Look for the kickoff comment ("Build started"), progress updates, and the final build
    reflection. These are first-person testimony from the agent that did the work.
5. Check closed issues for this cycle's milestone:
    ```bash
    gh issue list --milestone "<milestone>" --state closed
    ```
6. Check open issues that were NOT completed:
    ```bash
    gh issue list --milestone "<milestone>" --state open
    ```

## Reflect

Present the build agent's testimony (from pitch issue comments) alongside each question. The agent's
progress updates and build reflection are a first-person account of what happened — use them to
ground the conversation and surface things the user may not have seen.

Walk through these questions with the user:

### What shipped?

- Which scopes were completed?
- What was the final shape vs. the original pitch? (Check the agent's build reflection for their
  take on this.)
- Were success criteria met?

### What was cut?

- What scope was hammered (cut to fit the time box)?
- Were cuts the right call in hindsight?
- Did the agent's progress updates flag scope cuts as they happened?

### What went well?

- What patterns, tools, or approaches worked effectively?
- What should we keep doing?
- What did the agent call out as easier than expected?

### What didn't go well?

- Where did we get stuck? Why?
- Were there rabbit holes the pitch didn't anticipate?
- Did the appetite feel right?
- What did the agent flag as harder than expected or as dead ends explored?

### What did we learn?

- New technical knowledge or patterns discovered
- Process improvements identified
- Domain insights gained
- What did the agent say future agents or future cycles should know?

## Capture Ideas

For each new idea, observation, or improvement surfaced during reflection:

1. Search for existing issues first: `gh issue list --search "<keywords>" --state all`
2. Read `{{ template_dir }}` to discover available issue types and their labels. Create each issue
   using the labels from the matching template:
    ```bash
    gh issue create --title "<idea>" --label "<labels from template>"
    ```
3. Present the captured issues to the user for review

## Record

Summarize the retrospective as a wiki page using the wiki skill:

- Page name: `Retro-Cycle-<N>.md` (stored in `{{ wiki_dir }}`)
- Include: what shipped, what was cut, key learnings, captured issue numbers
- Update `{{ wiki_home }}` to link the new page
