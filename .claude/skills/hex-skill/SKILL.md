---
name: hex-skill
description:
    Create, retrofit, or audit hex- skills to match project conventions. Use when writing a new
    skill, updating an existing skill to follow the assumptions and source-reference patterns, or
    checking skills for convention compliance. Also use when the user invokes /hex-skill.
---

# Skill

Create, retrofit, or audit hex- skills to match the project's skill conventions.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name              | Value                                     | Description                                           |
| ----------------- | ----------------------------------------- | ----------------------------------------------------- |
| `project_root`    | repository root                           | Base directory; all paths are relative to this        |
| `skill_guide`     | `{{ project_root }}/docs/guides/skill.md` | Convention rules for hex- skills                      |
| `skill_dir`       | `{{ project_root }}/.claude/skills`       | Directory containing all hex- skills                  |
| `exemplar_idea`   | `{{ skill_dir }}/hex-idea/SKILL.md`       | Reference exemplar for assumptions and source reading |
| `exemplar_commit` | `{{ skill_dir }}/hex-commit/SKILL.md`     | Reference exemplar for workflow structure             |

## 1. Learn the Conventions

Read `{{ skill_guide }}` to extract the project's current skill conventions. Hold in memory:

- **Assumptions table rules** — required columns, ordering, chaining syntax, the `project_root`
  requirement
- **Source material rules** — when to read vs hardcode, the "Read X to extract..." pattern, the
  source-of-truth table for common values
- **Voice rules** — imperative, no hedging, observable steps, describe WHAT not HOW to think
- **Structure rules** — frontmatter format, skeleton order, description conventions
- **Retrofitting checklist** — the six audit steps for existing skills

Also read the two exemplars (`{{ exemplar_idea }}` and `{{ exemplar_commit }}`) to see the
conventions applied in practice.

Do NOT hardcode convention rules — always read them fresh from `{{ skill_guide }}`.

## Which Workflow?

1. If the user wants to **create** a new skill → Create (below)
2. If the user wants to **update** or **retrofit** an existing skill → Retrofit (further below)
3. If the user wants to **audit** one or more skills without editing → Audit (bottom)

## Create

### 2. Understand the Domain

Ask the user what the skill does and when it should be invoked. Identify:

- What source materials the skill will reference (guides, templates, config files)
- What values the skill needs that are defined elsewhere
- What workflows the skill orchestrates

### 3. Build the Assumptions Table

For each source material and configurable value identified:

- Create an assumption with a clear name, the file path or value, and a description
- Chain assumptions where appropriate (`{{ project_root }}/...`)
- Start with `project_root` as the first row
- For values that originate in source files, point the assumption to the source file

### 4. Write the Skill

Follow the structure from `{{ skill_guide }}`:

- Frontmatter with triggering-conditions-only description ending with the invocation note
- One-line purpose statement
- Assumptions table with the introductory note about `{{ }}` syntax
- Workflow sections with numbered steps or named phases
- "Read `{{ assumption }}` to extract..." wherever the skill depends on external values

Write the skill to `{{ skill_dir }}/hex-<name>/SKILL.md`.

### 5. Validate

Walk through each convention rule from `{{ skill_guide }}` and verify the new skill complies:

- Every external value is in the assumptions table
- No hardcoded values that belong in source materials
- Voice matches conventions
- Structure matches skeleton
- Description is triggering conditions only with invocation note

## Retrofit

### 2. Read the Existing Skill

Read the skill from `{{ skill_dir }}/hex-<name>/SKILL.md`. Identify:

- Hardcoded file paths, labels, commands, and configurable values
- Values that are defined in source materials but encoded in the skill
- Voice deviations from conventions
- Structural deviations from the skeleton

### 3. Apply the Retrofitting Checklist

Follow the retrofitting checklist from `{{ skill_guide }}`:

1. Add Assumptions table
2. Replace hardcoded values with `{{ name }}` references
3. Add "Read `{{ file }}` to extract..." instructions
4. Adjust voice
5. Verify description
6. Verify structure

Present the proposed changes to the user before writing.

### 4. Write the Updated Skill

Apply the changes to `{{ skill_dir }}/hex-<name>/SKILL.md`.

## Audit

### 2. Select Scope

Ask the user: audit a single skill, or all hex- skills?

- **Single**: read `{{ skill_dir }}/hex-<name>/SKILL.md`
- **All**: list `{{ skill_dir }}/hex-*/SKILL.md` and read each

### 3. Check Each Skill

For each skill, check against every convention rule from `{{ skill_guide }}`. Report:

- **Pass**: convention followed
- **Violation**: what is wrong and what the fix would be

Present the audit as a table per skill with columns: Rule, Status, Notes.

### 4. Offer to Fix

If violations were found, offer to retrofit each skill using the Retrofit workflow above.
