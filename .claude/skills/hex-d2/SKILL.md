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

Check the first line of the `.d2` file for a render directive comment:

```
# render: <flags>
```

If present, extract the flags (e.g., `--pad 20`) and append them to the render command. If no render
directive is found, default to `--pad 5` — enough breathing room without the excessive whitespace of
D2's built-in 100px default.

```bash
{{ d2_command }} <flags> {{ diagram_dir }}/<name>.d2 {{ diagram_dir }}/<name>.svg
```

When creating a new diagram, include a render directive as the first line. Choose a pad value
appropriate for the embedding context — `--pad 0` for tight embedding, `--pad 5` for minimal
breathing room.

If the render fails, show the error output. Fix the D2 source and retry.

## 5. Return Paths

Report both paths to the caller:

- **D2 source**: `{{ diagram_dir }}/<name>.d2`
- **SVG output**: `{{ diagram_dir }}/<name>.svg`

Present the paths as both absolute and relative-to-`target_root` forms. Callers use these to:

- Embed the SVG in markdown: `![<alt>](<relative path to .svg>)`
- Reference the D2 source for readers: `<!-- diagram source: <relative path to .d2> -->`

## Quick Reference

Compact reference for common D2 patterns. For full documentation, see https://d2lang.com.

### Render Directive

D2 does not support in-file render options. This project uses a comment convention on the first line
of the `.d2` file to declare CLI flags:

```d2
# render: --pad 5
```

The skill parses this line and passes the flags to the `{{ d2_command }}` invocation. If no render
directive is present, the skill defaults to `--pad 5`.

### Padding

D2 defaults to `--pad 100`, adding 100px of whitespace on all sides of the diagram. This is almost
always excessive for diagrams embedded in markdown or wiki pages.

- `--pad 5` — project default; minimal breathing room (applied when no render directive is present)
- `--pad 0` — no outer whitespace
- `--pad 20` — moderate spacing if the diagram feels cramped

### Grid Layouts

D2 grids arrange children into rows or columns. Use nested grids for layouts that mix full-width
rows with multi-column sections.

**Basic structure:**

```d2
grid-rows: 3

header: {
  label: "Full-width header"
  shape: rectangle
}

middle: "" {
  grid-columns: 3

  sidebar: { label: "Left panel"; shape: rectangle }
  canvas: { label: "Center"; shape: rectangle }
  panel: { label: "Right panel"; shape: rectangle }
}

footer: {
  label: "Full-width footer"
  shape: rectangle
}
```

**Key patterns:**

- `grid-rows` on the outer container, `grid-columns` on nested containers
- `grid-gap: 0` — no spacing between grid cells
- **Empty label trick**: `name: "" {` prevents the container name from rendering as a visible label.
  Without the empty string, D2 displays the key name (e.g., "middle") as text.
- **Cell sizing**: Set `height` proportional to content. Oversized heights (e.g., `height: 180` for
  two lines of text) create disproportionate whitespace. Two lines of 14-18px text need roughly
  60-80px of height.
- `style.fill: transparent` and `style.stroke: transparent` on wrapper containers to make them
  invisible
- **Reserved keywords**: `left`, `right`, `top`, `bottom`, `center` are reserved by D2 for
  positioning. Do not use them as element keys — they cause compile errors. Use descriptive names
  like `sidebar`, `panel`, `canvas` instead.

### Labels

D2 supports string labels and markdown labels. They render differently in the SVG output.

**String labels** (recommended):

```d2
box: {
  label: "Line one\nLine two"
}
```

String labels render as SVG `<text>` elements with `<tspan>` for each line. They work in all SVG
viewers, browsers, and PNG converters.

**Markdown labels** (use with caution):

```d2
box: {
  label: |md
    # Heading
    Body text
  |
}
```

Markdown labels render inside `<foreignObject>` HTML elements in the SVG. These display correctly in
browsers but **fail to render** in PNG conversion tools (macOS `sips`, `rsvg-convert`). If the
diagram may be converted to PNG or viewed outside a browser, use string labels instead.

**Text wrapping**: D2 does not auto-wrap text within constrained cells. Use `\n` in string labels
for explicit line breaks. Without `\n`, long text overflows the cell boundary.
