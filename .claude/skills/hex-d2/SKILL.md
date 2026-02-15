---
name: hex-d2
description:
    Create or update D2 diagrams and render them to SVG. Use when a document, wiki page, or spec
    needs a new diagram or an existing diagram needs changes. Works with any target repository (main
    repo, wiki, or other). Also use when the user invokes /hex-d2.
---

# D2

Create or update a D2 diagram source file, render it to SVG, and return the paths so callers can
embed the result.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name                  | Value                          | Description                                                   |
| --------------------- | ------------------------------ | ------------------------------------------------------------- |
| `project_root`        | repository root                | Base directory; all paths are relative to this                |
| `mise_config`         | `{{ project_root }}/mise.toml` | Tool versions and task definitions — confirms d2 is available |
| `d2_command`          | `d2` (via mise)                | D2 CLI binary, activated through mise shims                   |
| `convention_primary`  | `docs/diagrams`                | Preferred diagram directory when a `docs/` folder exists      |
| `convention_fallback` | `diagrams`                     | Fallback diagram directory when no `docs/` folder exists      |

## 1. Identify the Target Repository

Determine which repository the diagram belongs to. The caller may specify a target explicitly, or it
can be inferred from context:

- **Main repo**: `{{ project_root }}`
- **Wiki repo**: `{{ project_root }}/.wiki`
- **Other**: any absolute path the caller provides

Set `target_root` to the resolved repository root for the rest of the workflow.

## 2. Locate the Diagrams Directory

Search for an existing diagrams directory within `target_root`:

1. List directories matching `**/diagrams` under `target_root` (non-recursive first: check
   `target_root/diagrams`, `target_root/docs/diagrams`).
2. If a `diagrams/` directory is found:
    - If its path matches `{{ convention_primary }}` or `{{ convention_fallback }}` relative to
      `target_root` → use it without prompting.
    - If it exists at a non-conventional path → present the found path to the user and ask for
      confirmation before using it.
3. If no `diagrams/` directory exists:
    - Check whether `target_root/docs/` exists.
    - If `docs/` exists → create `target_root/{{ convention_primary }}`.
    - If `docs/` does not exist → create `target_root/{{ convention_fallback }}`.

Set `diagram_dir` to the resolved directory for the rest of the workflow.

## 3. Create or Update the D2 File

Determine the diagram filename. The caller may provide a name, or derive one from context (e.g., the
topic being diagrammed). The filename must use kebab-case with no extension.

- **New diagram**: Write the `.d2` source to `{{ diagram_dir }}/<name>.d2`.
- **Existing diagram**: Read `{{ diagram_dir }}/<name>.d2`, apply the requested changes, and write
  it back.

## 4. Render to SVG

Run the D2 compiler to produce the SVG:

```bash
d2 {{ diagram_dir }}/<name>.d2 {{ diagram_dir }}/<name>.svg
```

If the render fails, show the error output. Fix the D2 source and retry.

## 5. Return Paths

Report both paths to the caller:

- **D2 source**: `{{ diagram_dir }}/<name>.d2`
- **SVG output**: `{{ diagram_dir }}/<name>.svg`

Present the paths as both absolute and relative-to-`target_root` forms. Callers use these to:

- Embed the SVG in markdown: `![<alt>](<relative path to .svg>)`
- Reference the D2 source for readers: `<!-- diagram source: <relative path to .d2> -->`
