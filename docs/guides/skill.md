# Hexorder — Skill Guide

## Purpose

This guide is the source of truth for how hex- skills are written and maintained. Every hex- skill
in `.claude/skills/` MUST follow these conventions. The `hex-skill` skill reads this guide at
runtime to enforce compliance.

---

## Assumptions Table

Every skill MUST have an **Assumptions** section immediately after the opening heading and purpose
line. The section contains a table declaring every external value the skill references.

### Table Format

| Column      | Purpose                                                                |
| ----------- | ---------------------------------------------------------------------- |
| Name        | Variable name used in `{{ name }}` references throughout the skill     |
| Value       | The resolved value — a path, label, or reference to another assumption |
| Description | What this value is and why it matters                                  |

### Rules

- All file paths, label names, command names, and configurable values MUST be declared as
  assumptions.
- Assumptions can reference other assumptions using `{{ name }}` syntax:
  `{{ project_root }}/docs/guides/git.md`.
- `project_root` MUST be the first assumption in every table, with value `repository root`.
- If a value originates in a source file (labels from issue templates, commit types from git.md,
  shaping steps from shape-up.md), the assumption points to **the source file** — not the value
  itself. The skill then reads the file to extract the value at runtime.
- The `{{ }}` delimiters indicate an assumption lookup. Include a brief note above the table
  explaining this for readers encountering the convention for the first time.

### Example

```markdown
## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name           | Value                                   | Description                                    |
| -------------- | --------------------------------------- | ---------------------------------------------- |
| `project_root` | repository root                         | Base directory; all paths are relative to this |
| `git_guide`    | `{{ project_root }}/docs/guides/git.md` | Commit format, types, scopes, and checklist    |
```

---

## Source Material References

Skills MUST read configuration from source files at runtime rather than encoding the values inline.

### Rules

- If a value is defined in another file (labels in issue templates, commit types in git.md, shaping
  steps in shape-up.md), the skill MUST read that file and extract the values.
- Skills MUST include an explicit instruction like **"Read `{{ file }}` to extract..."** followed by
  a description of what to look for.
- Skills MUST NOT hardcode lists, enums, formats, or structures that are defined in source
  materials. This ensures the skill adapts when source materials change.
- Skills MAY hardcode values that are intrinsic to the skill's own logic and not defined elsewhere.

### Source of Truth for Common Values

| Value Type        | Source of Truth                                      |
| ----------------- | ---------------------------------------------------- |
| Label names       | `.github/ISSUE_TEMPLATE/*.yml` (the `labels:` field) |
| Commit types      | `docs/guides/git.md`                                 |
| Commit scopes     | `docs/guides/git.md`                                 |
| Shaping steps     | `docs/guides/shape-up.md`                            |
| Plugin lifecycle  | `docs/guides/plugin.md`                              |
| Contract protocol | `docs/guides/contract.md`                            |
| Ship gate checks  | `CLAUDE.md` → Ship Gate                              |
| Hook config       | `lefthook.yml`                                       |

### Anti-Pattern

```markdown
<!-- BAD: hardcodes labels that live in issue templates -->

Search for issues with `status:triage`, `type:feature`, or `type:bug` labels.

<!-- GOOD: reads labels from source -->

Read `{{ template_dir }}` to discover available templates and their labels. Use the `labels:` field
from each template to determine which labels to search for.
```

---

## Voice

All hex- skills use the same voice.

### Rules

- **Imperative, agent-addressed**: "Read X", "Ask the user", "Run Y".
- **No hedging**: never "you might want to", "consider doing", or "it may be helpful to".
- **No personality**: no humor, metaphor, or conversational filler.
- **No first-person**: the skill is instructions, not a narrator.
- **Describe WHAT to do, not HOW to think**: "Search for duplicates" not "Think about whether there
  might be duplicates".
- **Trust agent judgment for unspecified details**: do not over-specify what the agent can infer
  from context.
- **Keep steps observable and verifiable**: each step should produce a visible result — a command
  run, a file read, a question asked, output displayed.

### Anti-Pattern

````markdown
<!-- BAD: hedging, personality, tells agent how to think -->

You might want to consider checking if there are any duplicate issues before creating a new one.
It's generally a good idea to search first!

<!-- GOOD: imperative, observable, direct -->

Search for existing issues that might cover the same ground:

```bash
gh issue list --search "<keywords>" --state all
```
````

````

---

## Structure

Every hex- skill follows this skeleton.

### Frontmatter

```yaml
---
name: hex-<name>
description:
    <Triggering conditions only. Starts with a verb. No workflow summary. Ends with:
    Also use when the user invokes /hex-<name>.>
---
````

Rules:

- `description` describes **WHEN** to use the skill, never **HOW** it works.
- Start with a verb phrase describing the triggering condition.
- End with "Also use when the user invokes /hex-\<name\>."
- Do not summarize the skill's workflow or steps in the description.

### Body

```markdown
# <Name>

<One-line purpose statement.>

## Assumptions

<Introductory note about {{ }} syntax> <Assumptions table>

## <Workflow sections — numbered steps or named phases>
```

Rules:

- The first section after the heading and purpose line is always **Assumptions**.
- Use **numbered steps** for linear flows (e.g., hex-commit: steps 1-10).
- Use **named phases** for branching flows (e.g., hex-cooldown: Phase 1-4).
- Include a **"Which Workflow?"** section when the skill has multiple entry paths (e.g., create vs
  retrofit vs audit).

---

## Retrofitting Checklist

When updating an existing skill to match these conventions:

1. **Add Assumptions table** — extract all file paths, labels, and configurable values into
   assumptions. Start with `project_root`.
2. **Replace hardcoded values** — swap inline values with `{{ name }}` references.
3. **Add source reading instructions** — wherever the skill currently encodes values from source
   materials, add "Read `{{ file }}` to extract..." instructions instead.
4. **Adjust voice** — rewrite hedging, personality, first-person, and "how to think" language to
   match the voice rules above.
5. **Verify description** — ensure frontmatter description is triggering conditions only, ends with
   the invocation note.
6. **Verify structure** — confirm the skeleton order: frontmatter, heading, purpose, assumptions,
   workflow sections.
