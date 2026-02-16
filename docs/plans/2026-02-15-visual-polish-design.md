# Design: Editor Visual Polish (#54)

## Open Questions

These need user input before implementation begins:

1. **Launcher coordination with #53 (Workspace lifecycle)** Pitch #53 changes the launcher screen
   significantly: it replaces the current "New Game System" button with an inline name-input flow,
   adds a "Close Project" menu item, and displays `Workspace.name` in the editor header bar
   (replacing or supplementing the "Hexorder" label + truncated ID). Pitch #54 also changes the
   launcher (amber heading, tagline, styled button) and the header bar (amber game system name).

    **Question**: What is the delivery order? Options:
    - (a) #54 lands first, #53 adapts the styled launcher. #54 would style the current launcher
      as-is (text mark, styled button, tagline), and #53 would later rework the launcher flow while
      preserving the visual polish.
    - (b) #53 lands first, #54 polishes the new launcher. #54 would then style the workspace-aware
      launcher (project name input, Create button, Workspace.name in header).
    - (c) Both build in parallel on separate branches against the integration branch. Merge
      conflicts in `systems.rs` (launcher_system, render_game_system_info) are resolved at
      integration time.

    **Recommendation**: Option (a) is cleanest if #54 is scoped to the current launcher. The
    launcher changes in #54 are purely visual (no structural additions), so #53 can preserve them
    when it rewrites the launcher flow. The header bar overlap is more concerning since #53 replaces
    the "Hexorder" + ID display with `Workspace.name`.

    **DECIDED**: Option (a) -- #54 lands first. #53 adapts the styled launcher.

2. **Header bar content: "Hexorder" label vs. Game System name in amber** The pitch says to apply
   amber to the "Game System name" in the header bar. Currently the header shows "Hexorder" (strong,
   15pt) + version + truncated ID. If #53 will replace this with `Workspace.name`, should #54 style
   the current "Hexorder" label in amber, or should we skip amber-on-header and let #53 define the
   header content?

    **DECIDED**: Style it now -- apply amber to the "Hexorder" label. #53 will preserve the amber
    styling when it adds `Workspace.name` alongside.

3. **`Color32::GRAY` and `Color32::WHITE` -- replace or keep?** The architecture test currently
   approves `Color32::GRAY` and `Color32::WHITE` as named constants. The pitch says to replace
   secondary labels' `Color32::GRAY` with the brand secondary text token `#808080`, and to replace
   `Color32::WHITE` swatch borders with amber. `Color32::GRAY` in egui is `(128, 128, 128)` -- close
   to but not exactly `#808080`. Should we:
    - (a) Replace all `Color32::GRAY` with explicit `from_gray(128)` (which IS `#808080`) and add
      128 to the approved grays. Result: visually identical, but explicit.
    - (b) Replace all `Color32::GRAY` with a `BrandTheme::TEXT_SECONDARY` constant that uses
      `from_gray(128)`.
    - (c) Leave `Color32::GRAY` as-is for secondary text (it is close enough to `#808080`).

    **Recommendation**: (b) for clarity. All color literals go through `BrandTheme`.

    **DECIDED**: Option (b) -- all `Color32::GRAY` and `Color32::WHITE` replaced through
    `BrandTheme` constants.

4. **Tab bar active indicator: amber text vs. amber underline?** The pitch says "use amber text or
   underline for the selected tab." Egui's `selectable_label` does not natively support underlines.
   Options:
    - (a) Set amber text color on the active tab label using `RichText::color()`.
    - (b) Custom paint an underline after the selectable_label. This is technically "new custom
      rendering," which the pitch's No Gos forbid.

    **Recommendation**: (a) amber text is simplest and stays within No Gos.

    **DECIDED**: Option (a) -- amber text on the active tab.

## Overview

This design covers a visual polish pass of the `editor_ui` plugin to align it with the brand
identity defined in `docs/brand.md`. The work introduces no new plugins, no new contracts, and no
structural layout changes. It is entirely internal to `editor_ui`.

### Scope Summary

| Element                  | Complexity | Description                                               |
| ------------------------ | ---------- | --------------------------------------------------------- |
| BrandTheme struct        | Low        | Named color constants, internal to editor_ui              |
| Text colors in Visuals   | Low        | Set fg_stroke on widget states in configure_theme         |
| Amber accent roll-out    | Medium     | Apply amber to ~14 specific UI sites                      |
| Font differentiation     | Low        | Change 4 TextStyle entries from Monospace to Proportional |
| Launcher polish          | Low        | Restyle text + button + add tagline                       |
| Panel spacing            | Low        | Width bump, add_space after headings, indent              |
| Architecture test update | Low        | Add new approved colors, remove old if needed             |

## BrandTheme Structure

The `BrandTheme` is **not** a Bevy `Resource`. It is a plain struct with `const` associated
constants in `src/editor_ui/components.rs`. It is never inserted into the ECS world -- it is
consumed directly by `configure_theme` and render functions at compile time.

```rust
// src/editor_ui/components.rs (or a new theme.rs sub-module)

use bevy_egui::egui;

/// Brand palette constants for the editor UI.
/// Source of truth: docs/brand.md
pub(crate) struct BrandTheme;

impl BrandTheme {
    // -- Backgrounds --
    /// Deep background (#0a0a0a) -- deepest UI panels
    pub const BG_DEEP: egui::Color32 = egui::Color32::from_gray(10);
    /// Panel fill (#191919) -- panel backgrounds
    pub const BG_PANEL: egui::Color32 = egui::Color32::from_gray(25);
    /// Surface (#232323) -- interactive surface areas / faint bg
    pub const BG_SURFACE: egui::Color32 = egui::Color32::from_gray(35);

    // -- Widget fills (graduated brightness for state) --
    pub const WIDGET_NONINTERACTIVE: egui::Color32 = egui::Color32::from_gray(30);
    pub const WIDGET_INACTIVE: egui::Color32 = egui::Color32::from_gray(40);
    pub const WIDGET_HOVERED: egui::Color32 = egui::Color32::from_gray(55);
    pub const WIDGET_ACTIVE: egui::Color32 = egui::Color32::from_gray(70);

    // -- Accent --
    /// Teal (#005c80) -- selection highlights, active states
    pub const ACCENT_TEAL: egui::Color32 = egui::Color32::from_rgb(0, 92, 128);
    /// Amber/gold (#c89640) -- emphasis, headings, primary actions
    pub const ACCENT_AMBER: egui::Color32 = egui::Color32::from_rgb(200, 150, 64);

    // -- Text --
    /// Primary text (#e0e0e0) -- body text, labels
    pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_gray(224);
    /// Secondary text (#808080) -- secondary labels, hints
    pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_gray(128);
    /// Disabled text (#505050) -- inactive elements
    pub const TEXT_DISABLED: egui::Color32 = egui::Color32::from_gray(80);
    /// Tertiary text -- used for IDs, de-emphasized metadata
    pub const TEXT_TERTIARY: egui::Color32 = egui::Color32::from_gray(120);

    // -- Border --
    /// Subtle border (#3c3c3c) -- panel borders, dividers
    pub const BORDER_SUBTLE: egui::Color32 = egui::Color32::from_gray(60);

    // -- Semantic --
    /// Danger (#c85050) -- destructive actions, error states
    pub const DANGER: egui::Color32 = egui::Color32::from_rgb(200, 80, 80);
    /// Success (#509850) -- valid states, confirmations
    pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(80, 152, 80);
}
```

### Design decisions

- **Not a Resource**: The theme is compile-time constants. No runtime overhead, no system parameter
  slot consumed, no initialization needed. This matches the pitch's "no theme hot-reloading" rabbit
  hole avoidance.
- **Struct with associated constants vs. module constants**: Associated constants on a struct
  provide namespacing (`BrandTheme::ACCENT_AMBER`) without polluting the module namespace. Either
  approach works; the struct approach reads more naturally.
- **`from_gray` values**: The brand doc defines panel fill as `#191919` (gray 25), surface as
  `#232323` (gray 35). The current code already uses these values -- they stay the same. The new
  additions are the text colors: gray 224 (primary), gray 128 (secondary), gray 80 (disabled).
- **TEXT_TERTIARY (gray 120)**: Currently used for the truncated ID label. Not in brand.md's palette
  table, but it is already in the approved architecture test. Kept as a named constant.

## Color Audit

Every `Color32` literal in `src/editor_ui/systems.rs` (production code only), mapped to its
BrandTheme equivalent:

### configure_theme (lines 32-41)

| Line | Current                             | BrandTheme Constant     | Change? |
| ---- | ----------------------------------- | ----------------------- | ------- |
| 32   | `from_gray(25)` -- panel_fill       | `BG_PANEL`              | Name    |
| 33   | `from_gray(25)` -- window_fill      | `BG_PANEL`              | Name    |
| 34   | `from_gray(10)` -- extreme_bg       | `BG_DEEP`               | Name    |
| 35   | `from_gray(35)` -- faint_bg         | `BG_SURFACE`            | Name    |
| 36   | `from_gray(30)` -- noninteractive   | `WIDGET_NONINTERACTIVE` | Name    |
| 37   | `from_gray(40)` -- inactive         | `WIDGET_INACTIVE`       | Name    |
| 38   | `from_gray(55)` -- hovered          | `WIDGET_HOVERED`        | Name    |
| 39   | `from_gray(70)` -- active           | `WIDGET_ACTIVE`         | Name    |
| 40   | `from_rgb(0, 92, 128)` -- selection | `ACCENT_TEAL`           | Name    |
| 41   | `from_gray(60)` -- window_stroke    | `BORDER_SUBTLE`         | Name    |

These are all name-only changes (values stay the same).

### Launcher (lines 74-96)

| Line | Current            | New                          | Change?       |
| ---- | ------------------ | ---------------------------- | ------------- |
| 74   | `"hexorder"` plain | `"HEXORDER"` amber color     | Text + color  |
| 79   | `Color32::GRAY`    | `TEXT_SECONDARY`             | Value (~same) |
| 84   | Button unstyled    | Amber text on primary button | New styling   |

### render_game_system_info (lines 280-301)

| Line | Current             | New               | Change?       |
| ---- | ------------------- | ----------------- | ------------- |
| 282  | `"Hexorder"` strong | Amber color added | Color         |
| 287  | `Color32::GRAY`     | `TEXT_SECONDARY`  | Value (~same) |
| 299  | `from_gray(120)`    | `TEXT_TERTIARY`   | Name only     |

### render_tool_mode (line 305)

| Line | Current              | New             | Change? |
| ---- | -------------------- | --------------- | ------- |
| 305  | `"Tool Mode"` strong | Add amber color | Color   |

### render_cell_palette (lines 365, 382)

| Line | Current                          | New             | Change?      |
| ---- | -------------------------------- | --------------- | ------------ |
| 365  | `"Cell Palette"` strong          | Add amber color | Color        |
| 382  | `Color32::WHITE` (swatch border) | `ACCENT_AMBER`  | Value change |

### render_unit_palette (lines 404, 421)

| Line | Current                          | New             | Change?      |
| ---- | -------------------------------- | --------------- | ------------ |
| 404  | `"Unit Palette"` strong          | Add amber color | Color        |
| 421  | `Color32::WHITE` (swatch border) | `ACCENT_AMBER`  | Value change |

### render_entity_type_section (line 485)

| Line | Current                | New             | Change? |
| ---- | ---------------------- | --------------- | ------- |
| 485  | `section_label` strong | Add amber color | Color   |

### Section headings (strong labels)

All section headings currently use `.strong()` only. Each gets `.color(BrandTheme::ACCENT_AMBER)`:

| Line | Heading          | Location                                              |
| ---- | ---------------- | ----------------------------------------------------- |
| 305  | "Tool Mode"      | `render_tool_mode`                                    |
| 365  | "Cell Palette"   | `render_cell_palette`                                 |
| 404  | "Unit Palette"   | `render_unit_palette`                                 |
| 485  | section_label    | `render_entity_type_section` (Cell Types, Unit Types) |
| 874  | "Enums"          | `render_enums_tab`                                    |
| 986  | "Structs"        | `render_structs_tab`                                  |
| 1110 | "Concepts"       | `render_concepts_tab`                                 |
| 1405 | "Relations"      | `render_relations_tab`                                |
| 1670 | "Constraints"    | `render_constraints_tab`                              |
| 1880 | "Validation"     | `render_validation_tab`                               |
| 1933 | "Inspector"      | `render_inspector`                                    |
| 2014 | "Unit Inspector" | `render_unit_inspector`                               |

### Secondary text (`Color32::GRAY` usages)

All `Color32::GRAY` usages in render functions replaced with `BrandTheme::TEXT_SECONDARY`:

| Line | Context                       |
| ---- | ----------------------------- |
| 79   | Launcher version text         |
| 287  | Header version text           |
| 573  | "(none)" property list        |
| 640  | "(comma-separated)" hint      |
| 890  | "(comma-separated)" enum hint |
| 918  | "No enums defined"            |
| 1015 | "No structs defined"          |
| 1143 | "No concepts defined"         |
| 1186 | Concept description           |
| 1195 | "(none)" roles                |
| 1269 | "(none)" bindings             |
| 1295 | Binding details               |
| 1587 | "No relations defined"        |
| 1697 | "No constraints defined"      |
| 1717 | Constraint expression         |
| 1851 | "(full editor coming soon)"   |
| 1937 | "No tile selected"            |
| 1973 | "No properties"               |
| 2018 | "No unit selected"            |
| 2166 | "(nested limit)"              |
| 2209 | "(nested limit)"              |
| 2246 | "(default)"                   |
| 2264 | "(nested limit)"              |
| 2303 | "(unknown struct)"            |

### Danger red (`from_rgb(200, 80, 80)` usages)

All mapped to `BrandTheme::DANGER`:

| Line | Context                 |
| ---- | ----------------------- |
| 851  | "Delete Type" button    |
| 965  | "Delete Enum" button    |
| 1088 | "Delete Struct" button  |
| 1383 | "Delete Concept" button |
| 1648 | "Delete" relation       |
| 1887 | Error count label       |
| 2070 | "Delete Unit" button    |

### Amber/gold (`from_rgb(200, 150, 64)` usages)

Already mapped to `BrandTheme::ACCENT_AMBER`:

| Line | Context                   |
| ---- | ------------------------- |
| 1706 | "[auto]" constraint badge |
| 1915 | Validation error category |

### Success green (`from_rgb(80, 152, 80)` usages)

Mapped to `BrandTheme::SUCCESS`:

| Line | Context        |
| ---- | -------------- |
| 1883 | "Schema Valid" |

### Other

| Line | Current          | New             | Change?   |
| ---- | ---------------- | --------------- | --------- |
| 299  | `from_gray(120)` | `TEXT_TERTIARY` | Name only |

## Text Colors

### Visuals Changes in configure_theme

Add explicit foreground stroke colors to widget states. Currently these inherit from
`Visuals::dark()` defaults. Brand palette defines:

- Primary text: `#e0e0e0` (224, 224, 224)
- Secondary text: `#808080` (128, 128, 128)
- Disabled text: `#505050` (80, 80, 80)

```rust
// In configure_theme, after existing widget bg_fill settings:

// Text colors (fg_stroke)
visuals.widgets.noninteractive.fg_stroke =
    egui::Stroke::new(1.0, BrandTheme::TEXT_PRIMARY);    // body text
visuals.widgets.inactive.fg_stroke =
    egui::Stroke::new(1.0, BrandTheme::TEXT_SECONDARY);  // inactive widget labels
visuals.widgets.hovered.fg_stroke =
    egui::Stroke::new(1.0, BrandTheme::TEXT_PRIMARY);    // hovered widget labels
visuals.widgets.active.fg_stroke =
    egui::Stroke::new(1.0, BrandTheme::TEXT_PRIMARY);    // active widget labels
visuals.widgets.open.fg_stroke =
    egui::Stroke::new(1.0, BrandTheme::TEXT_PRIMARY);    // open widget labels
```

Note: `widgets.noninteractive.fg_stroke` controls the color of `ui.label()` and `ui.heading()` text.
`widgets.inactive.fg_stroke` controls text on buttons/selectable labels in their default
(non-hovered, non-active) state. Setting inactive to secondary gives a natural dimmed look for
controls that are not being interacted with.

### Egui Dark Defaults (for reference)

Egui `Visuals::dark()` default text colors:

- `noninteractive.fg_stroke`: `(180, 180, 180)` -- our brand primary `(224, 224, 224)` is brighter
- `inactive.fg_stroke`: `(180, 180, 180)` -- we dim it to `(128, 128, 128)`
- `hovered.fg_stroke`: `(240, 240, 240)` -- our brand primary `(224, 224, 224)` is slightly dimmer
- `active.fg_stroke`: `(255, 255, 255)` -- our brand primary `(224, 224, 224)` is slightly dimmer

The change makes body text brighter and inactive controls dimmer compared to defaults. This creates
more visual hierarchy.

### Disabled widget text

Egui has a `widgets.disabled` state (in addition to noninteractive/inactive/hovered/active/open).
The `disabled` variant is controlled by `add_enabled_ui(false, ...)`. We should not override
`disabled.fg_stroke` to our disabled color because egui's disabled state already applies opacity.
Egui's disabled override will be sufficient.

### RichText Overrides

Explicit `.color()` calls on `RichText` override the Visuals defaults for specific elements. After
setting Visuals text colors, many current `.color(Color32::GRAY)` calls could theoretically be
removed (since the Visuals defaults would apply). However, these calls serve a purpose: they mark
text as deliberately secondary (hints, empty-state messages). Replacing them with
`BrandTheme::TEXT_SECONDARY` makes the intent explicit while staying consistent.

## Amber Accent Plan

### Elements receiving amber

| UI Element                    | Location (function, ~line)     | Current Color    | Change                                                  |
| ----------------------------- | ------------------------------ | ---------------- | ------------------------------------------------------- |
| Section headings (12 total)   | See "Section headings" above   | None (bold)      | Add `.color(BrandTheme::ACCENT_AMBER)`                  |
| Active tab label              | `render_tab_bar` L349          | Default          | Wrap active tab text in `RichText::color(ACCENT_AMBER)` |
| "New Game System" button      | `launcher_system` L84          | Default          | Use `RichText::color(ACCENT_AMBER)` for button text     |
| "+ Create" buttons            | Multiple create forms          | Default          | Use `RichText::color(ACCENT_AMBER)` for button text     |
| "+ Add" buttons               | Property/role/option add       | Default          | Use `RichText::color(ACCENT_AMBER)` for button text     |
| "+ Bind" buttons              | Concept binding                | Default          | Use `RichText::color(ACCENT_AMBER)` for button text     |
| Selected swatch border (cell) | `render_cell_palette` L382     | `Color32::WHITE` | `BrandTheme::ACCENT_AMBER`                              |
| Selected swatch border (unit) | `render_unit_palette` L421     | `Color32::WHITE` | `BrandTheme::ACCENT_AMBER`                              |
| Game System name in header    | `render_game_system_info` L282 | Default          | Add `.color(BrandTheme::ACCENT_AMBER)`                  |
| Launcher "HEXORDER" heading   | `launcher_system` L74          | Default          | Add `.color(BrandTheme::ACCENT_AMBER)`, uppercase       |

### Elements NOT receiving amber (per pitch scoping)

- "Open..." button on launcher -- secondary action, keep default
- "Save", "Save As" menu items -- keep default
- "x" delete buttons -- keep danger red
- Tab labels for inactive tabs -- keep default (only active tab gets amber)
- Form field labels ("Name:", "Color:") -- keep default body text

### Tab bar implementation detail

Current code at line 349-354:

```rust
if ui
    .selectable_label(editor_state.active_tab == tab, label)
    .clicked()
```

Changed to:

```rust
let text = if editor_state.active_tab == tab {
    egui::RichText::new(label).color(BrandTheme::ACCENT_AMBER)
} else {
    egui::RichText::new(label)
};
if ui.selectable_label(editor_state.active_tab == tab, text).clicked()
```

### Primary action buttons

The pitch calls out "+ Create", "+ Add", "+ Bind", "New Game System" as primary action buttons.
These are scattered across multiple render functions. Full list:

| Button Text             | Location (~line) | Function                     |
| ----------------------- | ---------------- | ---------------------------- |
| `"+ Create"` type       | 510              | `render_entity_type_section` |
| `"+ Add"` property      | 814              | `render_entity_type_section` |
| `"+ Create"` enum       | 905              | `render_enums_tab`           |
| `"+ Add"` enum option   | 957              | `render_enums_tab`           |
| `"+ Create"` struct     | 1001             | `render_structs_tab`         |
| `"+ Add"` struct field  | 1077             | `render_structs_tab`         |
| `"+ Create Concept"`    | 1130             | `render_concepts_tab`        |
| `"+ Add Role"`          | 1247             | `render_concepts_tab`        |
| `"+ Bind"`              | 1369             | `render_concepts_tab`        |
| `"+ Create Relation"`   | 1570             | `render_relations_tab`       |
| `"+ Create Constraint"` | 1862             | `render_constraints_tab`     |
| `"New Game System"`     | 84               | `launcher_system`            |

Each button text will be wrapped in `egui::RichText::new(...).color(BrandTheme::ACCENT_AMBER)`.

## Font Changes

### Current configure_theme Font Setup (lines 44-61)

```rust
style.text_styles.insert(
    egui::TextStyle::Heading,
    egui::FontId::new(20.0, egui::FontFamily::Monospace),
);
style.text_styles.insert(
    egui::TextStyle::Body,
    egui::FontId::new(15.0, egui::FontFamily::Monospace),
);
style.text_styles.insert(
    egui::TextStyle::Small,
    egui::FontId::new(13.0, egui::FontFamily::Monospace),
);
style.text_styles.insert(
    egui::TextStyle::Button,
    egui::FontId::new(15.0, egui::FontFamily::Monospace),
);
```

### New Font Setup

```rust
style.text_styles.insert(
    egui::TextStyle::Heading,
    egui::FontId::new(20.0, egui::FontFamily::Proportional),  // CHANGED
);
style.text_styles.insert(
    egui::TextStyle::Body,
    egui::FontId::new(15.0, egui::FontFamily::Proportional),  // CHANGED
);
style.text_styles.insert(
    egui::TextStyle::Small,
    egui::FontId::new(13.0, egui::FontFamily::Proportional),  // CHANGED
);
style.text_styles.insert(
    egui::TextStyle::Button,
    egui::FontId::new(15.0, egui::FontFamily::Proportional),  // CHANGED
);
style.text_styles.insert(
    egui::TextStyle::Monospace,
    egui::FontId::new(15.0, egui::FontFamily::Monospace),     // KEEP (explicit)
);
```

### What uses explicit FontFamily in render code?

There are **no** explicit `FontFamily::Monospace` or `FontFamily::Proportional` references in the
render functions -- all text goes through `RichText` which uses the default text style. This means
the font change is entirely contained in `configure_theme`. No render function changes needed for
fonts.

### Where should monospace be used?

Per the brand doc ("Monospace for data values and coordinates"), these displays should use
monospace:

| Element                     | Location                       | Current Font     | Action                         |
| --------------------------- | ------------------------------ | ---------------- | ------------------------------ |
| Position: (q, r)            | `render_inspector` L1941       | Body (was Mono)  | Add `.monospace()` to RichText |
| ID: truncated_id            | `render_game_system_info` L297 | Small (was Mono) | Add `.monospace()` to RichText |
| Property values (DragValue) | `render_property_value_editor` | Body (was Mono)  | Leave as-is (see below)        |
| Game System version         | Multiple locations             | Small (was Mono) | Add `.monospace()` to RichText |

The DragValue widget uses the Body text style internally. Since most property value editing happens
through egui widgets (DragValue, Checkbox, TextEdit), and those widgets pick up the Body style, we
have two options:

- (a) Leave DragValue as-is (will now be proportional). Monospace for numeric values is nice-to-have
  but not critical.
- (b) Use `ui.scope()` to temporarily override the Body style to Monospace around property editors.

**Recommendation**: Go with (a) for the initial pass. The pitch's rabbit hole section says "Do not
audit every RichText usage." If numeric values look wrong in proportional, it is a follow-up.

Specific monospace additions:

- `format!("Position: ({}, {})", pos.q, pos.r)` -- wrap in `RichText::new(...).monospace()`
- `format!("ID: {id_short}")` -- already `.small()`, add `.monospace()`
- `format!("v{}", gs.version)` -- keep `.small()`, add `.monospace()`
- `format!("v{}", env!("CARGO_PKG_VERSION"))` -- keep `.small()`, add `.monospace()`

## Launcher Changes

### Current Launcher (lines 64-98)

```
[30% vertical space]
"hexorder"                    -- 32pt monospace bold
"v0.8.0"                      -- small monospace gray
[24px space]
[New Game System]              -- 200x36 default button
[8px space]
[Open...]                      -- 200x36 default button
```

### New Launcher

```
[30% vertical space]
"HEXORDER"                     -- 32pt PROPORTIONAL bold, AMBER color
"Game System Design Tool"      -- small PROPORTIONAL, SECONDARY TEXT color (NEW)
[4px space]
"v0.8.0"                       -- small MONOSPACE, SECONDARY TEXT color
[24px space]
[New Game System]              -- 200x36, button text in AMBER
[8px space]
[Open...]                      -- 200x36, default styling (secondary action)
```

### Changes

1. `"hexorder"` becomes `"HEXORDER"` with `.color(BrandTheme::ACCENT_AMBER)`
2. Add tagline label: `"Game System Design Tool"` with `.small().color(BrandTheme::TEXT_SECONDARY)`
3. Version label: add `.monospace()`, replace `Color32::GRAY` with `BrandTheme::TEXT_SECONDARY`
4. "New Game System" button: wrap text in `RichText::new(...).color(BrandTheme::ACCENT_AMBER)`
5. "Open..." button: keep default styling (it is a secondary action)

### Coordination with #53

Pitch #53 will rework the launcher to include a project name input field and a "Create" button
instead of the current "New Game System" button. When #53 lands after #54:

- The "HEXORDER" heading, tagline, and version styling from #54 should be preserved.
- The inline name input + "Create" button from #53 should adopt amber styling for "Create".
- The "Open..." button should remain secondary.
- #53 should reference `BrandTheme::ACCENT_AMBER` and `BrandTheme::TEXT_SECONDARY` for its new UI
  elements.

If #53 lands first, #54 should apply the visual polish to whatever launcher state #53 creates.

## Spacing & Layout

### Sidebar width

```rust
// Line 152 -- change:
.default_width(260.0)
// To:
.default_width(280.0)
```

### Post-heading spacing

Add `ui.add_space(8.0)` after every section heading label (the 12 strong labels listed above). This
applies to standalone headings like `ui.label(RichText::new("Enums").strong())`, NOT to
`CollapsingHeader` headings (which have their own spacing).

Affected headings (non-collapsing):

- "Tool Mode" (L305) -- add `ui.add_space(8.0)` after
- "Cell Palette" (L365) -- add `ui.add_space(8.0)` after
- "Unit Palette" (L404) -- add `ui.add_space(8.0)` after
- "Enums" (L874) -- add `ui.add_space(8.0)` after
- "Structs" (L986) -- add `ui.add_space(8.0)` after
- "Concepts" (L1110) -- add `ui.add_space(8.0)` after
- "Relations" (L1405) -- add `ui.add_space(8.0)` after
- "Constraints" (L1670) -- add `ui.add_space(8.0)` after
- "Validation" (L1880) -- add `ui.add_space(8.0)` after

### Inter-section spacing

Increase existing `ui.add_space(4.0)` between form groups to `ui.add_space(6.0)` where they separate
logical sections. This is a judgment call during implementation -- not every 4.0 needs to change.

### Indentation

Add `ui.indent()` for sub-items in these contexts:

- Property lists under entity types (currently flat)
- Role lists under concepts (currently prefixed with " ")
- Binding lists under concepts (currently prefixed with " ")

Note: The pitch lists indentation as a spacing refinement. Adding `ui.indent()` does not constitute
"new widgets" (which is a No Go) -- it is a layout helper already available in egui.

### Separator consistency

Currently separators are used after: game system info, tool mode, tab bar, cell palette, unit
palette, and between tab content and inspector. No changes needed -- the current separator placement
is consistent.

## Architecture Test Updates

### Current approved palette (from main.rs test)

**Grays**: 10, 25, 30, 35, 40, 55, 60, 70, 120

**RGB**: (0,92,128), (200,80,80), (200,150,64), (80,152,80)

**Named**: `Color32::GRAY`, `Color32::WHITE`

### Changes needed

**Add to approved grays**:

- `224` -- `#e0e0e0` primary text (`TEXT_PRIMARY`), used in fg_stroke settings
- `128` -- `#808080` secondary text (`TEXT_SECONDARY`), replaces `Color32::GRAY`
- `80` -- `#505050` disabled text (`TEXT_DISABLED`), if used in Visuals

**Remove from approved named** (if all usages are replaced):

- `Color32::GRAY` -- replaced by `BrandTheme::TEXT_SECONDARY` (`from_gray(128)`)
- `Color32::WHITE` -- replaced by `BrandTheme::ACCENT_AMBER` (swatch borders)

**Important**: Before removing `Color32::GRAY` and `Color32::WHITE` from the approved list, verify
that ALL usages in non-test editor_ui files have been replaced. If any remain (e.g., in color
conversion utilities), keep them approved. The test already skips conversion utility functions.

### Updated approved palette

```rust
let approved_grays: &[u8] = &[
    10,  // #0a0a0a -- deep background (BG_DEEP)
    25,  // #191919 -- panel fill (BG_PANEL)
    30,  // widget noninteractive (WIDGET_NONINTERACTIVE)
    35,  // #232323 -- surface / faint bg (BG_SURFACE)
    40,  // widget inactive (WIDGET_INACTIVE)
    55,  // widget hovered (WIDGET_HOVERED)
    60,  // #3c3c3c -- border (BORDER_SUBTLE)
    70,  // widget active (WIDGET_ACTIVE)
    80,  // #505050 -- disabled text (TEXT_DISABLED) -- NEW
    120, // secondary label text (TEXT_TERTIARY)
    128, // #808080 -- secondary text (TEXT_SECONDARY) -- NEW
    224, // #e0e0e0 -- primary text (TEXT_PRIMARY) -- NEW
];

let approved_rgb: &[(u8, u8, u8)] = &[
    (0, 92, 128),   // #005c80 -- teal accent (ACCENT_TEAL)
    (200, 80, 80),  // #c85050 -- danger red (DANGER)
    (200, 150, 64), // #c89640 -- amber/gold accent (ACCENT_AMBER)
    (80, 152, 80),  // #509850 -- success green (SUCCESS)
];

// Named constants -- remove GRAY and WHITE if fully replaced
let approved_named: &[&str] = &[];
```

If `Color32::GRAY` or `Color32::WHITE` survive anywhere in non-test, non-conversion editor_ui code,
keep them in the approved list. The goal is zero unapproved colors, not necessarily zero named
constants.

## First Piece

**Build `BrandTheme` struct + `configure_theme` rewrite + architecture test update first.**

Rationale:

- It is the foundation that everything else depends on.
- It is the most core, novel piece (the struct definition establishes the vocabulary).
- It is small enough to complete in a few hours.
- It is end-to-end testable: `mise check:audit` (including the architecture test) must pass after
  this step.
- It surfaces any issues with `const` color construction in egui (e.g., `from_gray` vs `from_rgb`
  constness).

### Proposed build order

1. **BrandTheme + configure_theme + arch test** -- Foundation. Theme struct, rewrite configure_theme
   to use it, update the architecture test. Font changes happen here too (Monospace to
   Proportional). Run `mise check:audit`.

2. **Color literal replacement sweep** -- Replace all `Color32::GRAY`, `Color32::WHITE`,
   `from_gray(N)`, `from_rgb(R,G,B)` in render functions with `BrandTheme::` constants. This is
   mechanical find-and-replace. Run `mise check:audit`.

3. **Amber accent roll-out** -- Apply amber to section headings, active tab, primary buttons, swatch
   borders. Run app to verify visually.

4. **Launcher polish** -- Restyle the launcher screen (uppercase heading, tagline, amber button,
   monospace version). Run app to verify visually.

5. **Spacing changes** -- Width bump, post-heading spacing, indentation. Run app to verify visually.

6. **Monospace annotations** -- Add `.monospace()` to coordinate displays, IDs, versions. Run app to
   verify visually.

Each step is independently commitable and testable.

## Risk Assessment

### Low Risk

- **Font change from Monospace to Proportional**: This is a one-line change per TextStyle in
  `configure_theme`. Egui's default `Proportional` font is the system sans-serif. On macOS this is
  SF Pro or Helvetica. No font loading, no asset management.

- **Color constant replacement**: Purely mechanical. The values do not change (except `GRAY` which
  becomes explicit `from_gray(128)` -- visually identical). The architecture test catches any
  unapproved colors.

- **Architecture test update**: Adding approved values is additive. Removing approved named
  constants (`GRAY`, `WHITE`) only breaks if we miss a usage -- caught by the test itself.

### Medium Risk

- **Amber on section headings may reduce readability**: Amber on dark gray has lower contrast than
  white on dark gray. At 20pt heading size this should be fine, but smaller headings in collapsing
  headers (which use egui's default header styling, not our heading TextStyle) may need testing.
  Mitigation: verify visually after step 3.

- **Launcher coordination with #53**: If both pitches are in-progress simultaneously, merge
  conflicts in `launcher_system` and `render_game_system_info` are likely. Mitigation: resolve at
  integration time; the functions are small and conflicts will be straightforward.

- **egui_kittest test expectations**: The UI tests in `ui_tests.rs` use `get_by_label()` which
  matches on text content, not style. Changing "hexorder" to "HEXORDER" will break
  `game_system_info_shows_hexorder_label` (searches for "Hexorder"). Mitigation: update test
  expectations when making the text changes.

### Not a Risk

- **Theme hot-reloading**: Not being built (rabbit hole, explicitly avoided).
- **Custom font loading**: Not being built (system fonts only).
- **New contracts**: None introduced. `BrandTheme` is internal to `editor_ui`.
- **Cross-plugin impact**: Zero. All changes are within `src/editor_ui/`.
