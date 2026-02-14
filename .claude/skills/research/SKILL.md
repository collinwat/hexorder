---
name: research
description:
    Consume existing research from the GitHub Wiki or perform new research. Use when a build task
    needs domain context, when evaluating technology options, when a pitch references prior
    research, or when investigating unknowns before committing to an implementation. Also use when a
    GitHub Issue with type:research is assigned, or when the user invokes /research.
---

# Research

This skill supports two workflows: consuming existing research and performing new research.

## Consuming Existing Research

Research lives in the GitHub Wiki, cloned locally at `.wiki/`. If `.wiki/` is missing, run:

```bash
mise run wiki:clone
```

1. Read `.wiki/Research-Index.md` — this is the **single source of truth** for which research pages
   are relevant to which feature areas and topics
2. Read the relevant `.wiki/<Page-Name>.md` file(s) identified by the index
3. Read `.wiki/Home.md` for a complete list of all available research pages

### How to Use Findings

1. Read only the sections relevant to your current work (the Research Index has section pointers)
2. Summarize key findings that affect your implementation decisions
3. Reference the wiki page in your feature log entry
4. Do not copy research content into specs or code comments — reference the wiki page

## Performing New Research

### When to Research

- Before committing to an implementation approach for a novel problem
- When a pitch identifies an open question or unknown
- When existing research is outdated or doesn't cover a new area
- During cool-down when shaping future pitches

### Process

1. **Create a GitHub Issue** using the `research` template (`type:research` label). Define the
   question, context, and expected deliverables.

2. **Investigate.** Use web search, documentation, and source code analysis.

3. **Write the wiki page.** Follow this structure:

    ```markdown
    # <Title>

    ## Research Question

    > The specific question being investigated.

    ## Context

    Why this matters. What decision it unblocks.

    ## Findings

    [Organized by topic, source, or option as appropriate]

    ## Synthesis

    [Cross-cutting analysis, common patterns, trade-offs]

    ## Recommendation

    [Specific recommendation for Hexorder with rationale]
    ```

4. **Commit and push to the wiki:**

    ```bash
    cd .wiki
    git add <New-Page>.md
    # Update Home.md and Research-Index.md with the new entry
    git add Home.md Research-Index.md
    git commit -m "Add <topic> research"
    git pull --rebase
    git push
    cd ..
    ```

    If `git pull --rebase` surfaces conflicts, stop and resolve them with the user before pushing.

5. **Close the GitHub Issue.** Reference the wiki page URL in the closing comment.
