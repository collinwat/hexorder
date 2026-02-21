---
name: hex-cooldown
description:
    Run the cool-down protocol between build cycles. Orchestrates the retrospective, triage,
    shaping, and betting phases that transition from one cycle to the next. Use after a cycle ships
    (after the ship gate passes and cycle ship merge completes). Also use when the user invokes
    /hex-cooldown.
---

# Cooldown

The cool-down is the period between build cycles. It can be **full** (all four phases) or
**lightweight** (quick retro only), depending on whether the current cycle is part of a pre-bet
sequence.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name             | Value                                        | Description                                    |
| ---------------- | -------------------------------------------- | ---------------------------------------------- |
| `project_root`   | repository root                              | Base directory; all paths are relative to this |
| `shape_up_guide` | `{{ project_root }}/docs/guides/shape-up.md` | Batched Ceremonies section                     |
| `retro_skill`    | `/hex-retro`                                 | Retrospective skill                            |
| `triage_skill`   | `/hex-triage`                                | Triage skill                                   |
| `research_skill` | `/hex-research`                              | Research skill                                 |
| `pitch_skill`    | `/hex-pitch`                                 | Pitch shaping skill                            |
| `bet_skill`      | `/hex-bet`                                   | Betting table skill                            |

## Determine Ceremony Type

Read `{{ shape_up_guide }}` Batched Ceremonies section for the full criteria. Ask the user:

1. Is the next cycle already bet (part of a pre-bet sequence)?
2. Did the circuit breaker fire on the just-completed cycle?
3. Are there significant new issues or strategic shifts to address?

If the next cycle is pre-bet AND the circuit breaker did not fire AND no strategic reset is needed →
**lightweight ceremony**. Otherwise → **full ceremony**.

## Lightweight Ceremony

Post a lightweight retro comment on the cycle tracking issue:

```bash
gh issue comment <tracking-number> --body "$(cat <<'EOF'
## Lightweight retro

**Shipped:** <1-2 sentences on what shipped>
**Surprises/learnings:** <1-2 sentences>
**Issues captured:** <list issue numbers, or "none">

Proceeding to pre-bet cycle <next-version>.
EOF
)"
```

Then proceed directly to the next cycle's kickoff. No triage, shaping, or betting needed.

## Full Ceremony

### Phase 1: Retrospective

Run `{{ retro_skill }}` to reflect on the completed cycle. This surfaces learnings and captures new
ideas as GitHub Issues.

### Phase 2: Triage

Run `{{ triage_skill }}` to survey the full pool of open issues — raw ideas, deferred items, bugs,
tech debt. Identify clusters of related issues worth shaping. Close stale or duplicate issues.

### Phase 3: Shaping

For each cluster identified during triage:

1. Run `{{ research_skill }}` if the cluster has unknowns that need investigation first
2. Run `{{ pitch_skill }}` to shape the cluster into a formal pitch for the betting table

Not every cluster needs a pitch. Only shape ideas where the problem is real, the appetite is clear,
and you have a solution sketch. Ideas that are not shaped are left alone — important ones will
resurface.

### Phase 4: Betting

Run `{{ bet_skill }}` to review shaped pitches and decide what to build in the next cycle. When
batching, bet pitches for the next 2-3 cycles at once (see `{{ shape_up_guide }}` Batched
Ceremonies).
