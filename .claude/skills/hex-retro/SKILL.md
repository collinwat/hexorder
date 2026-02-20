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
| `plugins_dir`  | `{{ project_root }}/docs/plugins`           | Plugin spec and log directory                  |
| `template_dir` | `{{ project_root }}/.github/ISSUE_TEMPLATE` | Issue templates with type labels               |
| `wiki_dir`     | `.wiki`                                     | GitHub Wiki local clone                        |
| `wiki_home`    | `{{ wiki_dir }}/Home.md`                    | Wiki landing page                              |

## Gather Context

1. Extract the current cycle's bets from the milestone:
    ```bash
    gh issue list --milestone "<milestone>" --label "type:pitch" --state all
    ```
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

## Reflect — Agent's Account

Present the build agent's testimony (from pitch issue comments) as a summary. The agent's progress
updates and build reflection are a first-person account of what happened — use them to ground the
conversation and surface things the user may not have seen.

Summarize the agent's account under these headings. **Present this as a report, not a
conversation.** Do not ask questions yet — the user will respond in the next section.

### What shipped (agent's account)

- Which scopes were completed?
- What was the final shape vs. the original pitch? (From the agent's build reflection.)
- Were success criteria met?

### What was cut (agent's account)

- What scope was hammered (cut to fit the time box)?
- Did the agent's progress updates flag scope cuts as they happened?

### What went well (agent's account)

- What did the agent call out as effective patterns, tools, or approaches?
- What was easier than expected?

### What didn't go well (agent's account)

- Where did the agent get stuck? Why?
- Were there rabbit holes the pitch didn't anticipate?
- What was harder than expected or dead ends explored?

### What did the agent learn?

- Technical knowledge or patterns discovered
- Process improvements identified
- What did the agent say future agents or future cycles should know?

### Skill recommendations (agent's account)

- Did the agent identify repetitive multi-step workflows that would benefit from a dedicated skill?
- A skill candidate needs: 2+ use cases observed, non-deterministic decisions involved, and expected
  to recur in future cycles

After presenting the agent's account, proceed immediately to the Developer's Retrospective.

## Developer's Retrospective

> **GATE — Do not skip this section.** The agent's account is one perspective. The developer's
> perspective is equally important and often covers things the agent cannot see: process friction,
> tool ergonomics, collaboration quality, strategic direction, and the experience of working with
> agents. **You MUST ask the user for their feedback and wait for a response before proceeding to
> Capture Ideas.**

Ask the user the following questions. Use `AskUserQuestion` or direct prompting — the key
requirement is that the user responds before the workflow continues. Present all questions together
so the user can answer in one pass.

### Questions for the developer

1. **Your take on the cycle** — Do you agree with the agent's account? Anything it missed, got
   wrong, or understated?
2. **Process and workflow** — How did the agent collaboration itself go? Any friction in handoffs,
   communication, or the skills/tools used? Anything about the Shape Up process that felt off?
3. **What would you change?** — About the project structure, skills, agents, conventions, or
   development workflow. Things you noticed while watching the build that the agent wouldn't know.
4. **New ideas** — Features, improvements, research questions, or concerns that came to mind during
   the cycle — things not already captured in the agent's account.
5. **Anything else** — Open floor. Anything on your mind that doesn't fit the above.

### After the developer responds

- Integrate the developer's feedback with the agent's account
- Note disagreements or additions — these carry into Capture Ideas
- If the developer identified skill candidates, process/workflow issues, or structural changes,
  ensure they are captured as issues in the next section

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
