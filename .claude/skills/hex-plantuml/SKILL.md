---
name: hex-plantuml
description:
    Create or update PlantUML diagrams (including Salt wireframes and ditaa ASCII art) and render
    them to image files. Use when a document, wiki page, or spec needs a UML diagram, wireframe,
    spatial layout diagram, or any PlantUML-supported visualization. Also use when the user invokes
    /hex-plantuml.
---

# PlantUML

Create or update a PlantUML diagram source file, render it, and return the paths so callers can
embed the result.

## Assumptions

These values are referenced throughout the workflow using `{{ name }}` syntax. The `{{ }}`
delimiters indicate an assumption lookup. Assumptions can reference other assumptions. If the
project structure changes, update them here.

| Name                  | Value                   | Description                                              |
| --------------------- | ----------------------- | -------------------------------------------------------- |
| `project_root`        | repository root         | Base directory; all paths are relative to this           |
| `plantuml_command`    | `plantuml`              | PlantUML CLI binary                                      |
| `plantuml_install`    | `brew install plantuml` | Homebrew install command (includes Java dependency)      |
| `convention_primary`  | `docs/diagrams`         | Preferred diagram directory when a `docs/` folder exists |
| `convention_fallback` | `diagrams`              | Fallback diagram directory when no `docs/` folder exists |

## 1. Ensure PlantUML Is Available

Check whether `{{ plantuml_command }}` is on the PATH:

```bash
command -v {{ plantuml_command }}
```

- **If found**: Run `{{ plantuml_command }} -version` to confirm it works. Report the version to the
  user and proceed to step 2.
- **If not found**: PlantUML is not managed by mise and must be installed separately. Present the
  following to the user and ask for approval before proceeding:

    > PlantUML is not installed. It requires a Java runtime and the PlantUML JAR. The recommended
    > installation method is Homebrew:
    >
    > ```
    > {{ plantuml_install }}
    > ```
    >
    > This will install PlantUML and its Java dependency (OpenJDK) via Homebrew. Shall I run this?

    If the user approves, run `{{ plantuml_install }}`. After installation completes, run
    `{{ plantuml_command }} -version` and report what was installed (PlantUML version, Java runtime
    version) before proceeding.

    If the user declines or wants a different installation method, stop and let them handle it
    manually. Resume from step 2 once they confirm `{{ plantuml_command }}` is available.

## 2. Identify the Target Repository

Determine which repository the diagram belongs to. The caller may specify a target explicitly, or it
can be inferred from context:

- **Main repo**: `{{ project_root }}`
- **Wiki repo**: `{{ project_root }}/.wiki`
- **Other**: any absolute path the caller provides

Set `target_root` to the resolved repository root for the rest of the workflow.

## 3. Locate the Diagrams Directory

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

## 4. Choose the Diagram Type

Select the PlantUML dialect based on what is being diagrammed:

| Need                                                        | Dialect | Directive     | Output Format |
| ----------------------------------------------------------- | ------- | ------------- | ------------- |
| Form-like UI wireframe (buttons, inputs, dropdowns, panels) | Salt    | `@startsalt`  | SVG           |
| Spatial layout or ASCII art box diagram                     | ditaa   | `@startditaa` | **PNG only**  |
| Sequence, component, activity, class, or state diagram      | UML     | `@startuml`   | SVG           |

**When to use Salt vs ditaa:**

- **Salt** is best for interactive UI mockups — forms, toolbars, menus, trees, tabbed panels. It has
  widgets (buttons, checkboxes, dropdowns) and layout containers.
- **ditaa** is best for spatial arrangement diagrams — layouts with labeled regions, architecture
  boxes, network topology. It renders ASCII art directly into clean images. ditaa does NOT support
  SVG output — it always produces PNG regardless of the `-tsvg` flag. Using `-tsvg` with ditaa will
  produce a PNG file with an `.svg` extension, corrupting the output.

If the choice is unclear, ask the user.

## 5. Create or Update the .puml File

Determine the diagram filename. The caller may provide a name, or derive one from context (e.g., the
topic being diagrammed). The filename must use kebab-case with no extension.

- **New diagram**: Write the `.puml` source to `{{ diagram_dir }}/<name>.puml`.
- **Existing diagram**: Read `{{ diagram_dir }}/<name>.puml`, apply the requested changes, and write
  it back.

## 6. Render

Run the PlantUML compiler. The output format depends on the diagram type chosen in step 4.

**For Salt and UML** (SVG output):

```bash
{{ plantuml_command }} -tsvg {{ diagram_dir }}/<name>.puml
```

Output: `<name>.svg`

**For ditaa** (PNG output — SVG is not supported):

```bash
{{ plantuml_command }} -tpng {{ diagram_dir }}/<name>.puml
```

Output: `<name>.png`

**Validate the output.** After rendering, verify the output file exists and its format matches
expectations:

```bash
file {{ diagram_dir }}/<name>.<ext>
```

- SVG files should report as "SVG Scalable Vector Graphics image" or "XML"
- PNG files should report as "PNG image data"

If the format does not match (e.g., the `file` command reports PNG data for a `.svg` file), the
wrong render flag was used. Fix the render command and re-run.

If the render fails, show the error output. Fix the PlantUML source and retry.

## 7. Return Paths

Report both paths to the caller:

- **PlantUML source**: `{{ diagram_dir }}/<name>.puml`
- **Output image**: `{{ diagram_dir }}/<name>.<ext>` (`.svg` or `.png` depending on type)

Present the paths as both absolute and relative-to-`target_root` forms. Callers use these to:

- Embed in markdown: `![<alt>](<relative path to output>)`
- Reference the source for readers: `<!-- diagram source: <relative path to .puml> -->`

## Quick Reference

Compact syntax reference for the most common diagram types. For full documentation, see
https://plantuml.com.

### ditaa (ASCII Art Diagrams)

ditaa converts ASCII art box drawings into clean rendered images.

```
@startditaa
+-------------------------------------------------------------------+
|              Top Section Label                                     |
+------------+------------------------------+------------------------+
|            |                              |                        |
|  Left      |        CENTER                |  Right                 |
|  Panel     |       (dominant)             |  Panel                 |
|            |                              |                        |
+------------+------------------------------+------------------------+
|  Bottom Section Label                                             |
+-------------------------------------------------------------------+
@endditaa
```

**Formatting rules:**

- Corners use `+`, horizontal lines use `-`, vertical lines use `|`
- Text must be INSIDE boxes, not on border lines (text on a `+---+` line breaks rendering)
- Column widths are driven by the widest content in each column
- Rows are separated by `+---+---+` lines

**Common flags** (passed in the directive):

- `--no-shadows` — flat rendering, no drop shadows
- `--no-separation` — no padding between boxes
- `scale=1.5` — scale the output (useful for higher resolution)

Example with flags: `@startditaa(scale=1.5,--no-shadows)`

**Limitations:**

- Output is always PNG — the `-tsvg` flag is silently ignored
- No font styling (bold, italic) — text is rendered as-is
- Round corners are applied to standalone boxes by default

### Salt (Wireframes)

Salt is for interactive UI mockups with form widgets.

```
@startsalt
{
  Title text
  "Input field"  | "value       "
  ^Dropdown^
  [Button]
  ()  Unchecked radio
  (X) Checked radio
  []  Unchecked checkbox
  [X] Checked checkbox
  ---
  Separator above
}
@endsalt
```

**Layout containers** — the opening brace character controls the layout:

```
{     default vertical layout
{+    outer border with title
{#    grid with all lines
{!    grid with vertical lines only
{-    grid with horizontal lines only
{/    tabs          {/ Tab1 | Tab2 | Tab3 }
{*    menu          {* File | Edit | View }
{T    tree view     {T + Parent | ++ Child | +++ Grandchild }
{S    scroll area   {S scrollable content }
{^    group box with title
```

**Grid (table) example:**

```
@startsalt
{#
  . | Col 1 | Col 2
  Row 1 | val | val
  Row 2 | val | val
}
@endsalt
```

**Limitations:**

- No column spanning — every row in a `{#` grid has the same number of columns
- Column widths are driven by content — use padding text to force wider columns
- Separate `{#` blocks do not share column alignment
- Best for form-like layouts, not spatial arrangement diagrams (use ditaa instead)

### Sequence Diagrams

```
@startuml
participant Alice
participant Bob

Alice -> Bob: Request
activate Bob
Bob --> Alice: Response
deactivate Bob

note right of Bob: A note

alt success
  Bob -> Alice: OK
else failure
  Bob -> Alice: Error
end
@enduml
```

### Component Diagrams

```
@startuml
package "System" {
  [Component A] as A
  [Component B] as B
}

interface "API" as api

A - api
api --> B
@enduml
```

### Activity Diagrams

```
@startuml
start
:Step 1;
if (condition?) then (yes)
  :Action A;
else (no)
  :Action B;
endif
stop
@enduml
```
