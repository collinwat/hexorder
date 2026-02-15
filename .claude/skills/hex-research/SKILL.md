---
name: hex-research
description:
    Consume existing research from the GitHub Wiki or perform new research. Use when a build task
    needs domain context, when evaluating technology options, when a pitch references prior
    research, or when investigating unknowns before committing to an implementation. Also use when a
    GitHub Issue with a research label is assigned, or when the user invokes /hex-research.
---

# Research

Consume existing research from the GitHub Wiki or perform new research to support build decisions.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name             | Value                                        | Description                                                 |
| ---------------- | -------------------------------------------- | ----------------------------------------------------------- |
| `project_root`   | repository root                              | Base directory; all paths are relative to this              |
| `research_guide` | `{{ project_root }}/docs/guides/research.md` | Research process, wiki page structure, quality expectations |
| `wiki_index`     | `.wiki/Research-Index.md`                    | Research index — single source of truth for topic lookups   |
| `wiki_dir`       | `.wiki`                                      | GitHub Wiki local clone                                     |
| `template_dir`   | `{{ project_root }}/.github/ISSUE_TEMPLATE`  | Issue templates (for research type label)                   |
| `wiki_skill`     | `/hex-wiki`                                  | Wiki operations skill                                       |

## Which Workflow?

1. Read `{{ wiki_index }}` to check if relevant research already exists
2. If it does → **Consume** (below)
3. If it does not and the question warrants investigation → **Perform** (further below)

## Consuming Existing Research

1. Read `{{ wiki_index }}` — **single source of truth** for which research pages are relevant to
   which feature areas and topics
2. Read the relevant `{{ wiki_dir }}/<Page-Name>.md` file(s) identified by the index
3. Summarize key findings that affect your implementation decisions
4. Reference the wiki page in your feature log entry — do not copy content into specs or code

## Performing New Research

1. **Create a GitHub Issue.** Read `{{ template_dir }}` to discover the research issue template and
   its labels. Create the issue using the discovered template.

2. **Investigate.** Use web search, documentation, and source code analysis.

3. **Write the wiki page.** Read `{{ research_guide }}` to extract the expected wiki page structure.
   Write the page following that structure.

4. **Publish.** Use `{{ wiki_skill }}` to commit the new page, update Home.md and Research-Index.md,
   and push.

5. **Close the GitHub Issue.** Reference the wiki page URL in the closing comment.
