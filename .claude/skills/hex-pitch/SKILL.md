---
name: hex-pitch
description:
    Shape raw ideas into formal pitches for the betting table. Use during cool-down to transform
    GitHub Issues, observations, or fresh ideas into shaped proposals. Handles pitches built from
    existing issues, pitches created from scratch, or a mix of both. Also use when the user invokes
    /hex-pitch.
---

# Pitch

Shape raw ideas into formal pitches for the betting table.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name             | Value                                        | Description                                    |
| ---------------- | -------------------------------------------- | ---------------------------------------------- |
| `project_root`   | repository root                              | Base directory; all paths are relative to this |
| `shape_up_guide` | `{{ project_root }}/docs/guides/shape-up.md` | Shaping theory, steps, pitch ingredients       |
| `template_dir`   | `{{ project_root }}/.github/ISSUE_TEMPLATE`  | Issue templates with type and status labels    |
| `pitch_template` | `{{ template_dir }}/pitch.yml`               | Pitch template with fields and labels          |

## 1. Learn the Shaping Process

Read `{{ shape_up_guide }}` to extract the shaping steps, their structure, and the pitch
ingredients. Read `{{ pitch_template }}` to extract the pitch template fields and labels.

Hold this information in memory for the rest of the workflow. Do NOT hardcode shaping steps,
ingredients, or template fields — always read them fresh from the files.

## 2. Which Workflow?

1. Ask the user: are we shaping from existing GitHub Issues, from a fresh idea, or both?
2. If from issues → **Browse & Shape** (below)
3. If from a fresh idea → **Shape from Scratch** (further below)
4. If both → Browse first, then shape incorporating the issues

## Browse & Shape (from existing issues)

1. Read `{{ template_dir }}` to discover available issue types and their labels. Search for
   candidate issues using the discovered labels:
    ```bash
    gh issue list --state open --label "<discovered label>"
    gh issue list --search "<keywords>"
    ```
2. Present candidates to the user for selection
3. Read selected issues: `gh issue view <number>`
4. Proceed to **Shape the Pitch** below, incorporating the selected issues

## Shape from Scratch

1. Discuss the problem with the user — what pain point or opportunity?
2. Proceed to **Shape the Pitch** below

## Shape the Pitch

Walk through the shaping steps extracted from `{{ shape_up_guide }}`. The guide defines the
sequence, structure, and content expectations for each step. Work through them interactively with
the user.

When all shaping steps are complete, create the pitch Issue using `{{ pitch_template }}`:

```bash
gh issue create --template pitch.yml --title "<concise title>" \
  --label "<labels from template>" --label "area:<area>"
```

Fill in the ingredients defined in `{{ pitch_template }}`.

If shaping from existing issues, add them to the "Related raw ideas" field using `#number`
references.

## After the Pitch

The pitch is ready for the betting table. It will be reviewed during cool-down. If selected, it gets
assigned to a release milestone and enters the build cycle.
