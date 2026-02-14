# Hexorder — Research Guide

## Purpose

Research is investigation before commitment. It reduces risk by answering open questions before
implementation begins. Research outputs live in the GitHub Wiki, not in the main repo.

## Where Research Lives

All research documents live in the **GitHub Wiki**: https://github.com/collinwat/hexorder/wiki

The wiki is a separate git repo cloned locally at `.wiki/` (gitignored). Agents and developers read
pages directly from this directory. The wiki skill (`.claude/skills/wiki/SKILL.md`) provides
instructions for reading, writing, and pushing wiki content.

Research does NOT live in the main repo. The main repo contains code, specs, contracts, and process
docs. Research is reference material consumed during builds but not part of the build artifact.

### Finding Research

- `.wiki/Home.md` — lists all available research pages
- `.wiki/Research-Index.md` — maps feature areas and topics to specific research pages and sections

These two wiki pages are the **single source of truth** for what research exists and where to find
it. Do not duplicate their content elsewhere.

## Consuming Research During Builds

1. Read `.wiki/Research-Index.md` to find research relevant to your feature area
2. Read the relevant `.wiki/<Page-Name>.md` file
3. Summarize relevant findings in your feature log
4. Reference the wiki page when it informs a design decision

The research skill (`.claude/skills/research/SKILL.md`) provides step-by-step instructions for
agents.

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

4. **Push to the wiki.** Use the wiki skill to commit the new page, update Home.md and
   Research-Index.md, and push.

5. **Close the GitHub Issue.** Reference the wiki page URL in the closing comment.

### Quality Bar

Research documents should be:

- **Actionable** — lead to a decision or inform an implementation
- **Sourced** — include links to primary sources
- **Structured** — follow the standard template
- **Scoped** — answer a specific question, not survey everything
