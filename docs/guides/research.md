# Hexorder — Research Guide

## Purpose

Research is investigation before commitment. It reduces risk by answering open questions before
implementation begins. Research outputs live in the GitHub Wiki, not in the main repo.

## Where Research Lives

All research documents live in the **GitHub Wiki**: https://github.com/collinwat/hexorder/wiki

The wiki is a separate git repo cloned locally at `.wiki/` (gitignored). Agents and developers read
pages directly from this directory. If `.wiki/` is missing, run:

```bash
mise run wiki:clone
```

Research does NOT live in the main repo. The main repo contains code, specs, contracts, and process
docs. Research is reference material consumed during builds but not part of the build artifact.

## Current Research Pages

| Page                         | Topic                                                                     |
| ---------------------------- | ------------------------------------------------------------------------- |
| UI Architecture Survey       | Editor UI technology evaluation (egui, Qt, Dioxus, composable Rust stack) |
| Hex Wargame Reference Games  | Reference game catalog with licensing analysis                            |
| Hex Wargame Mechanics Survey | Comprehensive hex-and-counter wargame mechanics catalog                   |
| Game Engine Property Types   | Property type systems across game engines                                 |

## Feature-to-Research Lookup

| Feature Area   | Relevant Research                 |
| -------------- | --------------------------------- |
| `editor_ui`    | UI Architecture Survey            |
| `game_system`  | Reference Games, Property Types   |
| `hex_grid`     | Mechanics Survey                  |
| `ontology`     | Mechanics Survey                  |
| `rules_engine` | Mechanics Survey                  |
| `unit`         | Mechanics Survey, Reference Games |
| `cell`         | Mechanics Survey                  |
| `persistence`  | Property Types                    |
| `scripting`    | UI Architecture Survey            |

## Consuming Research During Builds

1. Check the lookup table above (or `.wiki/Research-Index.md` for section-level detail)
2. Read the relevant `.wiki/<Page-Name>.md` file
3. Summarize relevant findings in your feature log
4. Reference the wiki page when it informs a design decision

The research skill (`.claude/skills/research/SKILL.md`) provides the same lookup table and
step-by-step instructions for agents.

## Performing New Research

### When to Research

- **Before a build cycle**: When shaping a pitch, research informs the solution sketch and
  identifies rabbit holes
- **During orientation**: When starting a feature, check if relevant research already exists
- **When blocked**: When an implementation hits an unknown, research the options before committing
- **During cool-down**: Explore ideas for future pitches

Research is NOT required for every task. If the solution is well-understood and the patterns are
established, skip research and build.

### The Lifecycle

```
GitHub Issue (type:research) → Investigation → Wiki page → Consumed during builds
```

### Step by Step

1. **Create a GitHub Issue** using the `research` template. Define the question, context, and what
   "done" looks like.

2. **Investigate.** Use web search, documentation, source code, and domain references.

3. **Write the wiki page.** Follow the standard structure:
    - Research Question — the specific question being investigated
    - Context — why this matters, what decision it unblocks
    - Findings — organized by source, topic, or option
    - Synthesis — cross-cutting analysis, common patterns, trade-offs
    - Recommendation — specific to Hexorder, with rationale

4. **Push to the wiki.** Commit the new page in `.wiki/`, update Home.md and Research-Index.md,
   push.

5. **Close the GitHub Issue.** Reference the wiki page URL in the closing comment.

6. **Update references.** Add the new page to:
    - The wiki Home page and Research Index
    - The lookup table in this guide
    - The lookup table in `.claude/skills/research/SKILL.md`

### Quality Bar

Research documents should be:

- **Actionable** — lead to a decision or inform an implementation
- **Sourced** — include links to primary sources
- **Structured** — follow the standard template
- **Scoped** — answer a specific question, not survey everything
