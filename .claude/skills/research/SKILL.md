# Research

Use this skill when you need domain context for a build task, when exploring unknowns before
committing to an implementation, or when performing new research. This skill supports two workflows:
consuming existing research and performing new research.

## Consuming Existing Research

### When to Use

- Starting work on a feature and you need domain context
- Evaluating implementation options for a technology decision
- A pitch or spec references prior research

### Feature-to-Research Lookup

| Your Feature   | Wiki Page                                                     | What to Look For                                            |
| -------------- | ------------------------------------------------------------- | ----------------------------------------------------------- |
| `editor_ui`    | `UI-Architecture-Survey`                                      | Architecture options, framework comparison, test strategies |
| `game_system`  | `Hex-Wargame-Reference-Games`, `Game-Engine-Property-Types`   | Reference game rules, property type patterns                |
| `hex_grid`     | `Hex-Wargame-Mechanics-Survey`                                | Hex grid systems, coordinates, adjacency                    |
| `ontology`     | `Hex-Wargame-Mechanics-Survey`                                | Strategic systems, meta-mechanics, game concepts            |
| `rules_engine` | `Hex-Wargame-Mechanics-Survey`                                | Combat resolution, movement rules, supply/logistics         |
| `unit`         | `Hex-Wargame-Mechanics-Survey`, `Hex-Wargame-Reference-Games` | Counter properties, stacking, unit types                    |
| `cell`         | `Hex-Wargame-Mechanics-Survey`                                | Terrain types, hex properties, terrain effects              |
| `persistence`  | `Game-Engine-Property-Types`                                  | Serialization patterns across engines                       |
| `scripting`    | `UI-Architecture-Survey`                                      | mlua integration, embedded scripting survey                 |

### How to Read

Research lives in the GitHub Wiki, cloned locally at `.wiki/`. Read pages directly:

```
.wiki/UI-Architecture-Survey.md
.wiki/Hex-Wargame-Reference-Games.md
.wiki/Hex-Wargame-Mechanics-Survey.md
.wiki/Game-Engine-Property-Types.md
.wiki/Research-Index.md              # full topic index
```

If `.wiki/` is missing, clone it:

```bash
mise run wiki:clone
```

### How to Use Findings

1. Read only the sections relevant to your current work (use Research-Index.md for section pointers)
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
    git push
    cd ..
    ```

5. **Close the GitHub Issue.** Reference the wiki page URL in the closing comment.

6. **Update references.** Add the new page to:
    - The lookup table in this skill file
    - The lookup table in `docs/guides/research.md`
