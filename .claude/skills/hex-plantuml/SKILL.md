---
name: hex-plantuml
description:
    Create or update PlantUML diagrams (including Salt wireframes) and render them to SVG. Use when
    a document, wiki page, or spec needs a UML diagram, wireframe, or any PlantUML-supported
    visualization. Also use when the user invokes /hex-plantuml.
---

# PlantUML

Create or update a PlantUML diagram source file, render it to SVG, and return the paths so callers
can embed the result.

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

## 4. Create or Update the .puml File

Determine the diagram filename. The caller may provide a name, or derive one from context (e.g., the
topic being diagrammed). The filename must use kebab-case with no extension.

- **New diagram**: Write the `.puml` source to `{{ diagram_dir }}/<name>.puml`.
- **Existing diagram**: Read `{{ diagram_dir }}/<name>.puml`, apply the requested changes, and write
  it back.

## 5. Render to SVG

Run the PlantUML compiler to produce the SVG:

```bash
{{ plantuml_command }} -tsvg {{ diagram_dir }}/<name>.puml
```

PlantUML outputs `<name>.svg` in the same directory as the source file.

If the render fails, show the error output. Fix the PlantUML source and retry.

## 6. Return Paths

Report both paths to the caller:

- **PlantUML source**: `{{ diagram_dir }}/<name>.puml`
- **SVG output**: `{{ diagram_dir }}/<name>.svg`

Present the paths as both absolute and relative-to-`target_root` forms. Callers use these to:

- Embed the SVG in markdown: `![<alt>](<relative path to .svg>)`
- Reference the PlantUML source for readers: `<!-- diagram source: <relative path to .puml> -->`

## Quick Reference

Compact syntax reference for the most common diagram types. For full documentation, see
https://plantuml.com.

### Salt (Wireframes)

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

**Layout containers:**

- `{` — default vertical layout
- `{+` — outer border with title
- `{#` — grid with all lines
- `{!` — grid with vertical lines only
- `{-` — grid with horizontal lines only
- `{/` — tabs: `{/ Tab1 | Tab2 | Tab3 }`
- `{*` — menu: `{* File | Edit | View }`
- `{T` — tree view: `{T + Parent | ++ Child | +++ Grandchild }`
- `{S` — scroll area: `{S scrollable content }`
- `{^` — group box with title

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
