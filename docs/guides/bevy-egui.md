# bevy_egui 0.39 Developer Guide for Hexorder

> Canonical reference for bevy_egui 0.39 (egui 0.33) patterns, conventions, and pitfalls. Updated:
> 2026-02-08 | bevy_egui 0.39.1 (Bevy 0.18, egui 0.33)

---

## Table of Contents

1. [Quick Reference](#1-quick-reference)
2. [Plugin Setup](#2-plugin-setup)
3. [Context Access](#3-context-access)
4. [Scheduling](#4-scheduling)
5. [Layout Containers](#5-layout-containers)
6. [Widgets](#6-widgets)
7. [Input Passthrough](#7-input-passthrough)
8. [Styling and Theming](#8-styling-and-theming)
9. [Images and Textures](#9-images-and-textures)
10. [Side Panels with Viewport Adjustment](#10-side-panels-with-viewport-adjustment)
11. [Color Conversion](#11-color-conversion)
12. [Testing Patterns](#12-testing-patterns)
13. [Performance](#13-performance)
14. [Hexorder-Specific Conventions](#14-hexorder-specific-conventions)
15. [Common Pitfalls](#15-common-pitfalls)
16. [Deprecations & Migration](#16-deprecations--migration)

---

## 1. Quick Reference

### Version Compatibility

| Bevy | bevy_egui | egui      |
| ---- | --------- | --------- |
| 0.18 | 0.39      | 0.33      |
| 0.17 | 0.37-0.38 | 0.32-0.33 |
| 0.16 | 0.34-0.36 | 0.30-0.32 |

### Key Imports

```rust
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
// For input passthrough:
use bevy_egui::input::{egui_wants_any_pointer_input, egui_wants_any_keyboard_input};
// For images:
use bevy_egui::EguiTextureHandle;
// For settings:
use bevy_egui::{EguiGlobalSettings, EguiContextSettings};
```

### Minimal Example

```rust
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera3d::default());
        })
        .add_systems(EguiPrimaryContextPass, ui_system)
        .run();
}

fn ui_system(mut contexts: EguiContexts) -> Result {
    egui::Window::new("Hello").show(contexts.ctx_mut()?, |ui| {
        ui.label("world");
    });
    Ok(())
}
```

### Default Cargo Features

bevy_egui default features: `manage_clipboard`, `open_url`, `default_fonts`, `render`, `bevy_ui`,
`picking`.

---

## 2. Plugin Setup

### Basic Setup

```rust
app.add_plugins(EguiPlugin::default());
```

`EguiPlugin::default()` enables multi-pass mode (recommended). The single-pass option is deprecated.

### Plugin Deduplication

Bevy deduplicates plugins. It is safe to call `add_plugins(EguiPlugin::default())` even if another
plugin already added it. Hexorder's `EditorUiPlugin` does this for self-contained registration.

### Auto-Created Context

By default, bevy_egui automatically creates an `EguiContext` on the first camera spawned. To control
this manually:

```rust
fn setup(mut egui_global_settings: ResMut<EguiGlobalSettings>, mut commands: Commands) {
    egui_global_settings.auto_create_primary_context = false;
    commands.spawn((Camera3d::default(), PrimaryEguiContext));
}
```

### EguiGlobalSettings

```rust
#[derive(Resource)]
pub struct EguiGlobalSettings {
    pub auto_create_primary_context: bool,        // default: true
    pub enable_focused_non_window_context_updates: bool, // default: true
    pub input_system_settings: EguiInputSystemSettings,
    pub enable_absorb_bevy_input_system: bool,    // default: false
    pub enable_cursor_icon_updates: bool,          // default: true
    pub enable_ime: bool,                          // default: true
}
```

### EguiContextSettings (Per-Context)

Attached as a component to the egui context entity:

```rust
#[derive(Component)]
pub struct EguiContextSettings {
    pub run_manually: bool,    // default: false
    pub scale_factor: f32,     // default: 1.0
    pub capture_pointer_input: bool, // default: true (picking feature)
    pub input_system_settings: EguiInputSystemSettings,
    pub enable_cursor_icon_updates: bool, // default: true
    pub enable_ime: bool,      // default: true
}
```

Adjusting scale factor for HiDPI:

```rust
fn update_scale(egui_context: Single<(&mut EguiContextSettings, &Camera)>) {
    let (mut settings, camera) = egui_context.into_inner();
    settings.scale_factor = 1.0 / camera.target_scaling_factor().unwrap_or(1.0);
}
```

---

## 3. Context Access

### EguiContexts System Parameter

`EguiContexts` is a `SystemParam` that provides convenient access to the egui context:

```rust
fn my_ui(mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;  // Returns &mut egui::Context
    egui::Window::new("Panel").show(ctx, |ui| {
        ui.label("Hello");
    });
    Ok(())
}
```

**Important:** `ctx_mut()` returns `Result<&mut egui::Context, QuerySingleError>`. Systems using
`EguiContexts` should return `Result` and use `?`.

### Direct EguiContext Query

For advanced use cases (multi-window, manual context control):

```rust
fn my_ui(mut egui_ctx: Single<&mut EguiContext, With<PrimaryEguiContext>>) {
    egui::Window::new("Panel").show(egui_ctx.get_mut(), |ui| {
        ui.label("Hello");
    });
}
```

### Combining with Bevy Queries

UI systems can access any Bevy resource or query alongside `EguiContexts`:

```rust
fn editor_panel(
    mut contexts: EguiContexts,
    mut editor_tool: ResMut<EditorTool>,
    selected: Res<SelectedHex>,
    terrain_query: Query<&Terrain>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::SidePanel::left("editor").show(ctx, |ui| {
        // Use Bevy resources and queries inside the closure
        ui.label(format!("Selected: {:?}", selected.position));
    });
    Ok(())
}
```

---

## 4. Scheduling

### EguiPrimaryContextPass (Recommended)

All UI systems should run in the `EguiPrimaryContextPass` schedule:

```rust
app.add_systems(EguiPrimaryContextPass, my_ui_system);
```

This schedule runs inside Bevy's `PostUpdate` via `run_egui_context_pass_loop_system`. The egui
context is properly initialized with input and screen size before your systems run.

### Why Not Update?

Running egui systems in `Update` instead of `EguiPrimaryContextPass` may work in single-pass mode,
but:

- Single-pass mode is deprecated
- Multi-pass mode (default) requires `EguiPrimaryContextPass`
- Using `EguiPrimaryContextPass` is compatible with both modes

### System Ordering Within EguiPrimaryContextPass

Multiple UI systems in `EguiPrimaryContextPass` can be ordered with `.chain()` or `SystemSet`, just
like in other schedules:

```rust
app.add_systems(
    EguiPrimaryContextPass,
    (toolbar_system, palette_system, info_panel_system).chain(),
);
```

### Plugin System Sets (for hooking into bevy_egui internals)

| Set                                | Schedule     | Purpose                      |
| ---------------------------------- | ------------ | ---------------------------- |
| `EguiStartupSet::InitContexts`     | `PreStartup` | Primary context creation     |
| `EguiPreUpdateSet::InitContexts`   | `PreUpdate`  | Context init for new cameras |
| `EguiPreUpdateSet::ProcessInput`   | `PreUpdate`  | Input reading                |
| `EguiPreUpdateSet::BeginPass`      | `PreUpdate`  | Starts egui pass             |
| `EguiPostUpdateSet::EndPass`       | `PostUpdate` | Ends egui pass               |
| `EguiPostUpdateSet::ProcessOutput` | `PostUpdate` | Processes egui output        |

---

## 5. Layout Containers

Egui uses an immediate-mode layout model. Containers are shown in order: panels first, then central
panel, then windows.

### SidePanel

```rust
egui::SidePanel::left("my_left_panel")
    .default_width(200.0)
    .resizable(true)
    .show(ctx, |ui| {
        ui.heading("Tools");
        ui.separator();
        // widgets...
    });

egui::SidePanel::right("my_right_panel")
    .default_width(150.0)
    .show(ctx, |ui| {
        ui.label("Properties");
    });
```

### TopBottomPanel

```rust
egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
    egui::MenuBar::new().ui(ui, |ui| {
        egui::containers::menu::MenuButton::new("File").ui(ui, |ui| {
            if ui.button("Quit").clicked() { /* ... */ }
        });
    });
});

egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
    ui.label("Ready");
});
```

### CentralPanel

Fills remaining space after panels. Only one per context:

```rust
egui::CentralPanel::default().show(ctx, |ui| {
    ui.heading("Main Content");
});
```

### Window (Floating)

```rust
let mut is_open = true;
egui::Window::new("Properties")
    .open(&mut is_open)       // close button
    .resizable(true)
    .vscroll(true)             // vertical scrollbar
    .default_size([300.0, 400.0])
    .show(ctx, |ui| {
        ui.label("Window content");
    });
```

### ScrollArea

```rust
egui::ScrollArea::vertical()
    .max_height(200.0)
    .show(ui, |ui| {
        for i in 0..100 {
            ui.label(format!("Item {i}"));
        }
    });
```

### CollapsingHeader

```rust
egui::CollapsingHeader::new("Advanced Settings")
    .default_open(false)
    .show(ui, |ui| {
        ui.label("Hidden content");
    });
```

### Frame

```rust
egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
    // Custom drawing area
});

// Custom frame
egui::Frame::new()
    .inner_margin(8.0)
    .corner_radius(4.0)
    .fill(egui::Color32::from_gray(32))
    .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
    .show(ui, |ui| {
        ui.label("Framed content");
    });
```

### Layout Helpers

```rust
// Horizontal layout
ui.horizontal(|ui| {
    ui.label("Name:");
    ui.text_edit_singleline(&mut name);
});

// Vertical (default)
ui.vertical(|ui| {
    ui.label("Line 1");
    ui.label("Line 2");
});

// Right-to-left, bottom-up
ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
    ui.hyperlink("https://example.com");
});

// Group (draws a frame around content)
ui.group(|ui| {
    ui.label("Grouped content");
});

// Indent
ui.indent("indent_id", |ui| {
    ui.label("Indented");
});

// Columns
ui.columns(2, |columns| {
    columns[0].label("Left");
    columns[1].label("Right");
});
```

---

## 6. Widgets

### Labels and Text

```rust
ui.label("Simple text");
ui.heading("Section Title");
ui.monospace("fixed-width text");
ui.small("Fine print");

// Colored / rich text
ui.label(egui::RichText::new("Warning!").color(egui::Color32::RED).strong());
```

### Buttons

```rust
if ui.button("Click Me").clicked() {
    // handle click
}

// Sized button
if ui.add_sized([120.0, 40.0], egui::Button::new("Big Button")).clicked() {
    // ...
}
```

### Selectable Label

Toggle-style button useful for toolbars:

```rust
let is_active = *tool == EditorTool::Select;
if ui.selectable_label(is_active, "Select").clicked() {
    *tool = EditorTool::Select;
}
```

### Checkbox

```rust
ui.checkbox(&mut my_bool, "Enable feature");
```

### Slider and DragValue

```rust
ui.add(egui::Slider::new(&mut value, 0.0..=100.0).text("Speed"));
ui.add(egui::DragValue::new(&mut value).speed(0.1).range(0.0..=10.0));
```

### TextEdit

```rust
// Single line
ui.text_edit_singleline(&mut my_string);

// Multi-line
ui.text_edit_multiline(&mut my_string);

// With hint text
ui.add(egui::TextEdit::singleline(&mut my_string).hint_text("Type here..."));
```

### ComboBox (Dropdown)

```rust
egui::ComboBox::from_label("Terrain")
    .selected_text(format!("{:?}", selected_terrain))
    .show_ui(ui, |ui| {
        ui.selectable_value(&mut selected_terrain, TerrainType::Plains, "Plains");
        ui.selectable_value(&mut selected_terrain, TerrainType::Forest, "Forest");
        ui.selectable_value(&mut selected_terrain, TerrainType::Water, "Water");
    });
```

### Color Swatch (Custom)

No built-in color picker in base egui; paint a filled rect:

```rust
let (rect, response) = ui.allocate_exact_size(
    egui::vec2(20.0, 20.0),
    egui::Sense::click(),
);
if ui.is_rect_visible(rect) {
    ui.painter().rect_filled(rect, 2.0, color);
    if is_selected {
        ui.painter().rect_stroke(
            rect, 2.0,
            egui::Stroke::new(2.0, egui::Color32::WHITE),
            egui::StrokeKind::Outside,
        );
    }
}
if response.clicked() {
    // handle selection
}
```

### Separator and Spacing

```rust
ui.separator();                              // horizontal line
ui.add_space(10.0);                          // vertical gap
ui.allocate_space(egui::Vec2::new(1.0, 20.0)); // explicit spacer
```

### Hyperlink

```rust
ui.hyperlink("https://example.com");
ui.add(egui::Hyperlink::from_label_and_url("Click here", "https://example.com"));
```

### Image

```rust
ui.add(egui::widgets::Image::new(egui::load::SizedTexture::new(
    texture_id,
    [256.0, 256.0],
)));
```

### Response Handling

Every widget returns a `Response`:

```rust
let response = ui.button("Click");

response.clicked()          // primary button click this frame
response.secondary_clicked() // right-click
response.double_clicked()   // double-click
response.hovered()          // mouse over (and not occluded)
response.changed()          // value changed (sliders, text edits)
response.lost_focus()       // widget lost keyboard focus
response.gained_focus()     // widget gained keyboard focus
response.dragged()          // being dragged
response.drag_delta()       // Vec2 of drag movement

// Tooltip
response.on_hover_text("This is a tooltip");

// Context menu (right-click)
response.context_menu(|ui| {
    if ui.button("Delete").clicked() { /* ... */ }
});
```

### Sense Types

```rust
egui::Sense::click()           // responds to clicks
egui::Sense::drag()            // responds to drags
egui::Sense::click_and_drag()  // both
egui::Sense::hover()           // only hover detection
```

---

## 7. Input Passthrough

Preventing game input when interacting with UI is critical for editor tools.

### Hexorder Strategy: Run Conditions + Absorb

Hexorder uses **both** approaches together:

1. **Run conditions** on game input systems (pointer and keyboard)
2. **`enable_absorb_bevy_input_system = true`** for egui text input to work

Run conditions alone are **not sufficient** for text input. They only prevent your custom systems
from running — Bevy's internal input systems still consume keyboard events before egui can process
them. Without absorb enabled, typing into `text_edit_singleline` fields produces no characters.

```rust
// In plugin build():
app.world_mut()
    .resource_mut::<EguiGlobalSettings>()
    .enable_absorb_bevy_input_system = true;
```

### Option A: Run Conditions (For Game Input Systems)

```rust
use bevy_egui::input::{egui_wants_any_pointer_input, egui_wants_any_keyboard_input};

app.add_systems(
    Update,
    handle_hex_click.run_if(not(egui_wants_any_pointer_input)),
)
.add_systems(
    Update,
    keyboard_shortcuts.run_if(not(egui_wants_any_keyboard_input)),
);
```

These run conditions check the `EguiWantsInput` resource (updated in `PostUpdate`):

| Method                       | Returns true when...                            |
| ---------------------------- | ----------------------------------------------- |
| `is_pointer_over_area()`     | Mouse is over any egui area                     |
| `wants_pointer_input()`      | Egui is interested in the pointer               |
| `is_using_pointer()`         | Egui is actively using pointer (e.g., dragging) |
| `wants_keyboard_input()`     | Egui is listening for text input                |
| `is_popup_open()`            | A context menu or popup is open                 |
| `wants_any_pointer_input()`  | Any of the pointer conditions above             |
| `wants_any_keyboard_input()` | keyboard input OR popup open                    |
| `wants_any_input()`          | Any input at all                                |

### Option B: Absorb Input (Required for Text Fields)

```rust
// Enable in plugin build — required for egui text input to receive keystrokes
app.world_mut()
    .resource_mut::<EguiGlobalSettings>()
    .enable_absorb_bevy_input_system = true;
```

This clears Bevy's input buffers when egui wants input. Must be enabled if your UI has text fields
(`text_edit_singleline`, `text_edit_multiline`, `TextEdit`). Without it, Bevy's internal systems
consume keyboard events before egui processes them.

### Option C: bevy_picking Integration

With the default `picking` feature enabled, `bevy_egui` automatically suppresses `bevy_picking`
events when the pointer is over egui windows. No additional code needed.

### EguiWantsInput Resource (Direct Access)

```rust
fn my_system(egui_wants: Res<bevy_egui::input::EguiWantsInput>) {
    if egui_wants.wants_any_pointer_input() {
        return; // skip game input handling
    }
}
```

---

## 8. Styling and Theming

### Setting Visuals

```rust
fn configure_visuals(mut contexts: EguiContexts) -> Result {
    contexts.ctx_mut()?.set_visuals(egui::Visuals {
        window_corner_radius: 0.0.into(),
        ..egui::Visuals::dark()   // or ::light()
    });
    Ok(())
}
```

### Visuals Structure (Key Fields)

```rust
egui::Visuals {
    dark_mode: true,
    override_text_color: None,                    // Option<Color32>
    window_corner_radius: egui::CornerRadius::same(6),
    window_shadow: egui::Shadow::NONE,
    window_fill: egui::Color32::from_gray(27),
    window_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
    panel_fill: egui::Color32::from_gray(27),
    faint_bg_color: egui::Color32::from_gray(35),
    extreme_bg_color: egui::Color32::from_gray(10),
    selection: egui::style::Selection {
        bg_fill: egui::Color32::from_rgb(0, 92, 128),
        stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(192, 222, 255)),
    },
    hyperlink_color: egui::Color32::from_rgb(90, 170, 255),
    ..Default::default()
}
```

### Widget Visuals (Per-State)

```rust
let mut visuals = egui::Visuals::dark();
visuals.widgets.inactive.bg_fill = egui::Color32::from_gray(40);
visuals.widgets.hovered.bg_fill = egui::Color32::from_gray(60);
visuals.widgets.active.bg_fill = egui::Color32::from_gray(80);
ctx.set_visuals(visuals);
```

Widget states: `noninteractive`, `inactive`, `hovered`, `active`, `open`.

### Scoped Style Changes

```rust
ui.scope(|ui| {
    ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
    ui.label("This is red");
});
ui.label("This is normal");
```

### Fonts

```rust
fn configure_fonts(mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;
    let mut fonts = egui::FontDefinitions::default();
    // Add custom font
    fonts.font_data.insert(
        "my_font".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!("path/to/font.ttf"))),
    );
    fonts.families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());
    ctx.set_fonts(fonts);
    Ok(())
}
```

---

## 9. Images and Textures

### Displaying Bevy Images in Egui

```rust
// 1. Load the image as a Bevy asset
let image_handle: Handle<Image> = asset_server.load("textures/icon.png");

// 2. Register it with egui (strong handle = egui shares ownership)
let texture_id = contexts.add_image(EguiTextureHandle::Strong(image_handle.clone()));

// 3. Display it
ui.add(egui::widgets::Image::new(egui::load::SizedTexture::new(
    texture_id,
    [64.0, 64.0],
)));
```

### Weak vs Strong Handles

```rust
// Strong: egui co-owns the asset (prevents unloading)
contexts.add_image(EguiTextureHandle::Strong(handle.clone()));

// Weak: egui does NOT own it (asset can be unloaded externally)
contexts.add_image(EguiTextureHandle::Weak(handle.id()));
```

Use **weak** when you manage asset lifetime yourself. Use **strong** when the image is only used in
egui.

### Removing Images

```rust
contexts.remove_image(&image_handle);
```

### Egui-Native Textures

```rust
let texture_handle = ctx.load_texture(
    "my-texture",
    egui::ColorImage::example(),
    Default::default(),
);
ui.image(egui::load::SizedTexture::new(
    texture_handle.id(),
    texture_handle.size_vec2(),
));
```

### Premultiplied Alpha

bevy_egui 0.39.1 fixed text AA by no longer re-premultiplying alpha for egui textures. If you load
images for display in egui, you may need to premultiply alpha manually:

```rust
fn premultiply(image: &mut Image) {
    for x in 0..image.width() {
        for y in 0..image.height() {
            let mut color = image.get_color_at(x, y).unwrap().to_linear();
            color.red *= color.alpha;
            color.green *= color.alpha;
            color.blue *= color.alpha;
            image.set_color_at(x, y, Color::LinearRgba(color)).unwrap();
        }
    }
}
```

---

## 10. Side Panels with Viewport Adjustment

For editor UIs where side panels should not overlap the 3D viewport, adjust the camera viewport
based on panel sizes:

```rust
fn editor_ui(
    mut contexts: EguiContexts,
    mut camera: Single<&mut Camera, Without<EguiContext>>,
    window: Single<&mut Window, With<PrimaryWindow>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let mut left = egui::SidePanel::left("tools")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Tools");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response.rect.width();

    // Scale from logical to physical units
    left *= window.scale_factor();

    let pos = UVec2::new(left as u32, 0);
    let size = UVec2::new(window.physical_width(), window.physical_height())
        - pos;

    camera.viewport = Some(Viewport {
        physical_position: pos,
        physical_size: size,
        ..default()
    });

    Ok(())
}
```

**Note:** When using a separate egui camera (as in the side_panel example), disable
`auto_create_primary_context` and spawn the egui camera with `PrimaryEguiContext` on a separate
render layer.

---

## 11. Color Conversion

Bevy and egui use different color types. Convert between them:

### Bevy Color to egui Color32

```rust
fn bevy_color_to_egui(color: Color) -> egui::Color32 {
    match color {
        Color::Srgba(c) => egui::Color32::from_rgba_unmultiplied(
            (c.red * 255.0) as u8,
            (c.green * 255.0) as u8,
            (c.blue * 255.0) as u8,
            (c.alpha * 255.0) as u8,
        ),
        Color::LinearRgba(c) => {
            let srgba: bevy::color::Srgba = c.into();
            egui::Color32::from_rgba_unmultiplied(
                (srgba.red * 255.0) as u8,
                (srgba.green * 255.0) as u8,
                (srgba.blue * 255.0) as u8,
                (srgba.alpha * 255.0) as u8,
            )
        }
        _ => egui::Color32::GRAY,
    }
}
```

### bevy_egui Helper Functions

The `bevy_egui::helpers` module provides:

```rust
use bevy_egui::helpers::{
    vec2_into_egui_pos2,    // bevy Vec2 -> egui Pos2
    vec2_into_egui_vec2,    // bevy Vec2 -> egui Vec2
    rect_into_egui_rect,    // bevy Rect -> egui Rect
    egui_pos2_into_vec2,    // egui Pos2 -> bevy Vec2
    egui_vec2_into_vec2,    // egui Vec2 -> bevy Vec2
    egui_rect_into_rect,    // egui Rect -> bevy Rect
};
```

---

## 12. Testing Patterns

### Testing State and Resources (Without Rendering)

Egui systems are hard to test with `app.update()` because they require the full render pipeline.
Instead, test the **state** that UI systems read and write:

```rust
#[test]
fn editor_tool_defaults_to_select() {
    let tool = EditorTool::default();
    assert_eq!(tool, EditorTool::Select);
}

#[test]
fn editor_tool_resource_inserts_correctly() {
    let mut app = App::new();
    app.insert_resource(EditorTool::default());
    app.update();
    let tool = app.world().resource::<EditorTool>();
    assert_eq!(*tool, EditorTool::Select);
}
```

### Testing UI Logic Separately

Extract logic from UI closures into testable functions:

```rust
// In systems.rs: pure function
fn format_terrain_type(tt: TerrainType) -> String {
    match tt {
        TerrainType::Plains => "Plains".to_string(),
        TerrainType::Forest => "Forest".to_string(),
        // ...
    }
}

// In tests.rs: test the logic
#[test]
fn terrain_formats_correctly() {
    assert_eq!(format_terrain_type(TerrainType::Plains), "Plains");
    assert_eq!(format_terrain_type(TerrainType::Forest), "Forest");
}
```

### Testing With Headless Egui Context

For integration-level tests that need an egui context:

```rust
#[test]
fn test_ui_layout() {
    let ctx = egui::Context::default();
    let output = ctx.run(egui::RawInput::default(), |ctx| {
        egui::SidePanel::left("test_panel").show(ctx, |ui| {
            ui.label("Test");
        });
    });
    // Inspect output.shapes, output.platform_output, etc.
    assert!(!output.shapes.is_empty());
}
```

---

## 13. Performance

### Immediate Mode Redraws

Egui rebuilds the entire UI every frame. For most editor UIs this is fine (1-2ms per frame).

### Guidelines

- **Avoid huge scroll areas**: Layout cost is proportional to content, even if not visible. Only lay
  out what's in view.
- **Minimize allocations**: Reuse `String` buffers via `Local<T>` instead of creating new strings
  each frame.
- **Conditional sections**: Use `CollapsingHeader` or conditionals to skip layout for hidden
  sections.
- **egui repaints only on interaction**: In idle state, egui skips repainting. However, Bevy still
  runs the system each frame.

### Bindless Mode

bevy_egui 0.39 supports bindless rendering (default array size: 16). This reduces bind group
switches when multiple textures are used:

```rust
// Configured in EguiPlugin (rarely needs changing)
EguiPlugin {
    bindless_mode_array_size: std::num::NonZero::new(16),
    ..default()
}
```

---

## 14. Hexorder-Specific Conventions

### UI System Pattern

All Hexorder egui systems follow this signature pattern:

```rust
pub fn my_panel_system(
    mut contexts: EguiContexts,
    // Bevy resources and queries...
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::SidePanel::left("panel_id").show(ctx, |ui| {
        // UI code using Bevy data
    });
}
```

**Note:** Hexorder uses `let Ok(ctx) = contexts.ctx_mut() else { return; }` rather than `-> Result`
with `?`. Both patterns are valid. The `else { return }` pattern avoids the `Result` return type on
the system.

### Plugin Registration

Each plugin registers its own egui systems:

```rust
impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default()); // safe to duplicate
        app.insert_resource(EditorTool::default());
        app.add_systems(EguiPrimaryContextPass, editor_panel_system);
    }
}
```

### Input Passthrough

Hexorder should use **run conditions** on game input systems:

```rust
// In hex_grid or camera plugin:
app.add_systems(
    Update,
    handle_hex_click.run_if(not(egui_wants_any_pointer_input)),
);
app.add_systems(
    Update,
    camera_pan.run_if(not(egui_wants_any_pointer_input)),
);
app.add_systems(
    Update,
    keyboard_shortcuts.run_if(not(egui_wants_any_keyboard_input)),
);
```

### Color Swatches

For terrain palette display, use `allocate_exact_size` with `Sense::click()` and paint directly:

```rust
let (rect, response) = ui.allocate_exact_size(
    egui::vec2(20.0, 20.0),
    egui::Sense::click(),
);
if ui.is_rect_visible(rect) {
    ui.painter().rect_filled(rect, 2.0, color);
    if is_active {
        ui.painter().rect_stroke(
            rect, 2.0,
            egui::Stroke::new(2.0, egui::Color32::WHITE),
            egui::StrokeKind::Outside,
        );
    }
}
```

---

## 15. Common Pitfalls

### 1. Systems Must Run in EguiPrimaryContextPass

```rust
// WRONG: UI won't render in multi-pass mode
app.add_systems(Update, my_ui_system);

// RIGHT:
app.add_systems(EguiPrimaryContextPass, my_ui_system);
```

### 2. ctx_mut() Returns Result

```rust
// WRONG: will panic
let ctx = contexts.ctx_mut().unwrap();

// RIGHT: handle gracefully
let Ok(ctx) = contexts.ctx_mut() else { return; };
// Or use -> Result with ?
let ctx = contexts.ctx_mut()?;
```

### 3. Panel IDs Must Be Unique

Each panel/window needs a unique string ID. Duplicate IDs cause layout conflicts:

```rust
// WRONG: both panels fight for the same ID
egui::SidePanel::left("panel").show(ctx, |ui| { /* A */ });
egui::SidePanel::left("panel").show(ctx, |ui| { /* B */ });

// RIGHT:
egui::SidePanel::left("tools_panel").show(ctx, |ui| { /* A */ });
egui::SidePanel::left("properties_panel").show(ctx, |ui| { /* B */ });
```

### 4. Panel Order Matters

Panels must be shown before `CentralPanel`. Side panels and top/bottom panels claim space first:

```rust
// RIGHT order:
egui::TopBottomPanel::top("menu").show(ctx, |ui| { /* ... */ });
egui::SidePanel::left("tools").show(ctx, |ui| { /* ... */ });
egui::CentralPanel::default().show(ctx, |ui| { /* ... */ }); // last
```

### 5. Input Passthrough Not Automatic

Clicking on egui panels will still trigger Bevy input systems unless you add run conditions:

```rust
// Without this, clicking a button will also click hex tiles behind it
app.add_systems(Update, handle_click.run_if(not(egui_wants_any_pointer_input)));
```

### 6. Scale Factor for Physical Coordinates

When converting egui panel sizes to Bevy viewport coordinates, multiply by `window.scale_factor()`:

```rust
let panel_width_logical = panel_response.rect.width();
let panel_width_physical = panel_width_logical * window.scale_factor();
```

### 7. Mutable Borrows in Closures

You cannot borrow `EguiContexts` mutably inside an egui closure because the context is already
borrowed. Extract the context first:

```rust
// WRONG: double mutable borrow
egui::Window::new("W").show(contexts.ctx_mut()?, |ui| {
    let tex_id = contexts.add_image(/*...*/); // ERROR
});

// RIGHT: extract context, then use it
let texture_id = contexts.add_image(EguiTextureHandle::Strong(handle.clone()));
let ctx = contexts.ctx_mut()?;
egui::Window::new("W").show(ctx, |ui| {
    ui.image(egui::load::SizedTexture::new(texture_id, [64.0, 64.0]));
});
```

### 8. Bevy Camera Required

bevy_egui requires at least one camera entity to render. If no camera exists, the egui context won't
be created and `ctx_mut()` will return `Err`.

### 9. Feature Flags Affect API

Some API is gated behind features:

- `EguiContexts::add_image` requires `render` feature (default)
- `capture_pointer_input` in `EguiContextSettings` requires `picking` feature (default)
- `EguiClipboard` requires `manage_clipboard` feature (default)

### 10. Text Input Requires Absorb

Egui `text_edit_singleline` and `text_edit_multiline` fields will silently fail to receive keyboard
input unless `enable_absorb_bevy_input_system` is `true`. Run conditions
(`egui_wants_any_keyboard_input`) only guard your custom systems — Bevy's internal input systems
still consume keyboard events. Always enable absorb in any plugin that uses egui text fields:

```rust
app.world_mut()
    .resource_mut::<EguiGlobalSettings>()
    .enable_absorb_bevy_input_system = true;
```

### 11. Multi-Pass Discards

In multi-pass mode, egui may run your UI closure multiple times per frame (for layout convergence).
Don't perform side effects (like sending events) inside egui closures — do them after the `.show()`
call based on captured state:

```rust
let mut should_fire_event = false;

egui::Window::new("W").show(ctx, |ui| {
    if ui.button("Fire").clicked() {
        should_fire_event = true; // capture, don't fire here
    }
});

if should_fire_event {
    commands.trigger(MyEvent); // fire outside the closure
}
```

### 12. egui Deprecations

See [§16 Deprecations & Migration](#16-deprecations--migration) for the full table.

---

## 16. Deprecations & Migration

> **Living document** — this section is updated as new deprecations are encountered. If you hit an
> egui deprecation not listed here, add it to the table before committing your fix.

| Deprecated API                      | Replacement                       | egui Version | Notes                  |
| ----------------------------------- | --------------------------------- | ------------ | ---------------------- |
| `ctx.screen_rect()`                 | `ctx.content_rect()`              | 0.33         | Method renamed         |
| `egui::Frame::none()`               | `egui::Frame::NONE`               | 0.33         | Constructor → const    |
| `egui::Margin::symmetric(f32, f32)` | `egui::Margin::symmetric(i8, i8)` | 0.33         | Parameter type changed |

### Code Examples

```rust
// DEPRECATED → REPLACEMENT

// screen_rect → content_rect
let screen = ctx.content_rect();

// Frame::none() → Frame::NONE
let frame = egui::Frame::NONE;

// Margin::symmetric takes i8, not f32
let margin = egui::Margin::symmetric(8, 4);
```

### Contribution Protocol

When you encounter an egui deprecation not in this table:

1. Fix the deprecation in your code
2. Add a row to the table above (deprecated API, replacement, egui version, notes)
3. Add a code example if the migration is non-obvious
4. Commit the guide update alongside your code fix
