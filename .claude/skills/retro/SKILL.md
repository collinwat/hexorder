---
name: retro
description:
    Run a cycle retrospective to reflect on what shipped, what was learned, and capture new ideas as
    GitHub Issues. Use at the end of a build cycle during cool-down, before shaping and betting for
    the next cycle. Also use when the user invokes /retro.
---

# Retro

Run a structured retrospective at the end of a build cycle. The goal is reflection and idea capture
— not commitments. New ideas become GitHub Issues for potential future shaping.

## Gather Context

1. Read `docs/coordination.md` — what was bet this cycle?
2. Review the cycle's git history:
    ```bash
    git log --oneline --since="<cycle-start>" --until="<cycle-end>"
    ```
3. Read plugin logs for plugins worked on this cycle: `docs/plugins/<name>/log.md`
4. Check closed issues for this cycle's milestone:
    ```bash
    gh issue list --milestone "<milestone>" --state closed
    ```
5. Check open issues that were NOT completed:
    ```bash
    gh issue list --milestone "<milestone>" --state open
    ```

## Reflect

Walk through these questions with the user:

### What shipped?

- Which scopes were completed?
- What was the final shape vs. the original pitch?
- Were success criteria met?

### What was cut?

- What scope was hammered (cut to fit the time box)?
- Were cuts the right call in hindsight?

### What went well?

- What patterns, tools, or approaches worked effectively?
- What should we keep doing?

### What didn't go well?

- Where did we get stuck? Why?
- Were there rabbit holes the pitch didn't anticipate?
- Did the appetite feel right?

### What did we learn?

- New technical knowledge or patterns discovered
- Process improvements identified
- Domain insights gained

## Capture Ideas

For each new idea, observation, or improvement surfaced during reflection:

1. Search for existing issues first: `gh issue list --search "<keywords>" --state all`
2. If no duplicate, create a new issue with the appropriate template:
    ```bash
    gh issue create --title "<idea>" --label "status:triage" --label "type:<type>"
    ```
    Types: `feature`, `bug`, `tech-debt`, `research`
3. Present the captured issues to the user for review

## Record

Summarize the retrospective as a wiki page using the wiki skill:

- Page name: `Retro-Cycle-<N>.md`
- Include: what shipped, what was cut, key learnings, captured issue numbers
- Update `.wiki/Home.md` to link the new page
