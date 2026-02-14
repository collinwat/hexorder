---
name: research
description:
    Consume existing research from the GitHub Wiki or perform new research. Use when a build task
    needs domain context, when evaluating technology options, when a pitch references prior
    research, or when investigating unknowns before committing to an implementation. Also use when a
    GitHub Issue with type:research is assigned, or when the user invokes /research.
---

# Research

For process details, rationale, and quality expectations, see `docs/guides/research.md`.

## Consuming Existing Research

1. Read `.wiki/Research-Index.md` — **single source of truth** for which research pages are relevant
   to which feature areas and topics
2. Read the relevant `.wiki/<Page-Name>.md` file(s) identified by the index
3. Summarize key findings that affect your implementation decisions
4. Reference the wiki page in your feature log entry — do not copy content into specs or code

## Performing New Research

1. **Create a GitHub Issue** using the `research` template (`type:research` label)

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

4. **Publish.** Use the wiki skill to commit the new page, update Home.md and Research-Index.md, and
   push.

5. **Close the GitHub Issue.** Reference the wiki page URL in the closing comment.
