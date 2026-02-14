# Design Tool UI Architecture & Test Driver Survey

## Research Prompt

> Survey how major game engines and design tool platforms build their editor UI (form-based panels +
> GPU 3D viewports) and test UI interactions. Evaluate alternative architectures for Hexorder's
> editor — specifically the tension between rich form-based UI and native GPU rendering with
> shaders.

## Date

2026-02-13

## Context

Hexorder currently uses Bevy 0.18 + bevy_egui for its editor UI. While egui works well for quick
panels, it becomes painful for complex form-heavy editors (ontology editors, rule builders, property
inspectors). This research surveys how industry tools solve the same problem and evaluates
alternative architectures for Hexorder.

---

## The Core Tension

A design tool like Hexorder needs two things that live in different ecosystems:

1. **Rich form UI** — property editors, rule builders, data tables, dropdowns, validation
2. **Native GPU rendering** — 3D hex grid, shaders, real-time simulation visualization

No single Rust framework does both exceptionally well today.

---

## Industry Survey

### 1. Unity — Custom GPU-Rendered UI

**Architecture:**

- C++ core engine
- Custom **IMGUI** (immediate-mode) for the original editor UI
- Transitioning to **UI Toolkit** — a retained-mode system modeled after web tech (USS stylesheets ~
  CSS, UXML templates ~ HTML, C# event handling ~ JS)
- Everything is GPU-rendered — the inspector fields, hierarchy tree, dropdown menus, and 3D scene
  view all go through Unity's rendering pipeline
- The 3D viewport is a **render texture** composited into the UI layout, not a separate window

**Form UI approach:**

- Started with IMGUI (similar to egui), hit the same pain points with complex forms
- Built UI Toolkit as a retained-mode replacement inside their own renderer
- UI Toolkit has a queryable element tree (like a DOM)

**Testing:**

- **Unity Test Framework** (NUnit-based) runs inside the engine process
- Edit Mode tests and Play Mode tests
- **Input simulation** via `InputTestFixture`
- **UQuery** for finding UI Toolkit elements by name/class/type and triggering events
  programmatically
- No standardized external driver protocol
- Third-party tools fill the gap: **AltTester** (injects instrumentation, exposes RPC for external
  test scripts), **Airtest/Poco** (similar injection + external driver)

**Limitations:** No external test driver protocol. Everything runs in-process. External UI testing
requires third-party SDK injection.

---

### 2. Autodesk (Maya, 3ds Max, AutoCAD) — Qt + Embedded Scripting

**Architecture:**

- C++ core engine for rendering/simulation
- **Qt (PySide)** for all form UI — panels, property editors, node editors, outliner
- Embedded **Python** and legacy languages (MEL for Maya, MAXScript for 3ds Max, AutoLISP for
  AutoCAD) with full access to both UI and engine
- The 3D viewport is a **Qt widget hosting an OpenGL/Vulkan surface** — same window, same process

**Form UI approach:**

- Qt provides mature, production-grade form widgets natively
- Property editors, node graphs, and tool panels are all standard Qt widgets
- Qt's signal/slot system handles form validation and data binding

**Testing:**

- The embedded scripting language **is** the test driver — no external protocol needed
- Python scripts running inside Maya can:
    - Create and manipulate UI widgets (`maya.cmds`)
    - Set field values, trigger button clicks, query widget state
    - Drive the engine (create objects, run simulations, assert state)
    - Access Qt widgets directly via PySide
- Everything runs in-process with no serialization overhead
- Studios build entire test suites in Python that exercise both UI and engine logic

**Example (Maya Python):**

```python
import maya.cmds as cmds

window = cmds.window()
cmds.columnLayout()
field = cmds.textField()
cmds.textField(field, edit=True, text="test_value")
result = cmds.textField(field, query=True, text=True)
assert result == "test_value"

cmds.polyCube(name="test_cube")
assert cmds.objExists("test_cube")
```

**Key insight:** The scripting layer serves double duty — it's both a user-facing feature (artists
script workflows) and the test driver.

---

### 3. Unreal Engine — Custom Slate Framework + Reflection System

**Architecture:**

- C++ core engine
- **Slate** — a completely custom, platform-agnostic UI framework written from scratch in C++
- Retained-mode with a declarative C++ syntax using operator overloading
- Every pixel in the editor is drawn by Slate through Unreal's renderer
- The 3D viewport is a Slate widget hosting a GPU rendering surface — same window, same process

**Slate UI declaration:**

```cpp
SNew(SHorizontalBox)
+ SHorizontalBox::Slot()
  .AutoWidth()
  [
    SNew(STextBlock)
    .Text(FText::FromString("Terrain Type:"))
  ]
+ SHorizontalBox::Slot()
  [
    SNew(SEditableTextBox)
    .OnTextCommitted(this, &SMyPanel::OnNameChanged)
  ]
```

**Form UI approach — reflection-driven auto-generation:**

- Unreal's **UPROPERTY reflection system** auto-generates editor forms from code annotations
- The Details Panel reads UPROPERTY metadata and generates the correct widget automatically:
    - Clamped int/float → slider
    - Enum → dropdown
    - Color → color picker
    - Asset reference → asset selector
- Developers never manually build forms for data editing — they annotate data types and the editor
  generates the UI

**UPROPERTY example:**

```cpp
UPROPERTY(EditAnywhere, Category="Combat", meta=(ClampMin=0, ClampMax=100))
int32 AttackStrength;

UPROPERTY(EditAnywhere, Category="Combat", meta=(ToolTip="Damage type"))
EDamageType DamageType;  // auto-generates dropdown

UPROPERTY(EditAnywhere, Category="Visual")
FLinearColor UnitColor;  // auto-generates color picker
```

**Testing:**

- **Built-in Automation Framework** with `FAutomationTestBase`
- Tests run from the editor (Session Frontend > Automation tab) or command line
- **Slate's queryable widget tree** — find widgets by type, name, or hierarchy
- Simulate clicks, keystrokes, drag operations directly through `FSlateApplication`
- **Python scripting** (Editor Script Plugin) for driving editor operations
- **Gauntlet** — Epic's scalable test orchestration for headless CI runs

**Automation test example:**

```cpp
IMPLEMENT_SIMPLE_AUTOMATION_TEST(
    FMySlateTest,
    "Editor.UI.PropertyPanel.SetValue",
    EAutomationTestFlags::EditorContext
)

bool FMySlateTest::RunTest(const FString& Parameters)
{
    FSlateApplication::Get().ProcessMouseButtonDownEvent(/* ... */);
    TestEqual("Value updated", Widget->GetText(), ExpectedText);
    return true;
}
```

**Key insight:** The UPROPERTY reflection system is what makes Unreal's editor scale.
Auto-generating forms from annotated types eliminates most manual form UI work.

---

### 4. Blender — Custom GPU UI + Embedded Python

**Architecture:**

- C/C++ core engine
- Custom UI framework — GPU-renders everything (like Unity)
- Embedded **Python** with full access to every operator and UI element
- All UI is drivable from Python

**Testing:**

- Python scripting is the test driver (same as Autodesk model)
- Operators are the atomic units of both user actions and test steps

---

## Comparison Matrix

| Aspect               | Unity                                        | Autodesk (Maya)                   | Unreal                         | Blender                 |
| -------------------- | -------------------------------------------- | --------------------------------- | ------------------------------ | ----------------------- |
| **UI Framework**     | Custom IMGUI → UI Toolkit                    | Qt (third-party)                  | Slate (custom)                 | Custom GPU UI           |
| **Form Generation**  | Manual                                       | Manual                            | Auto-generated from reflection | Manual (Python-defined) |
| **GPU Viewport**     | Same renderer                                | Qt widget hosting GPU             | Slate widget hosting GPU       | Same renderer           |
| **Test Driver**      | Internal framework; third-party for external | Embedded Python drives everything | Built-in automation + Python   | Embedded Python         |
| **Engineering Cost** | Very high                                    | Medium (leverages Qt)             | Very high                      | High                    |
| **Rust Viability**   | N/A (C#)                                     | CXX-Qt for logic, C++ for widgets | N/A (C++)                      | N/A (C/C++)             |

---

## Alternative Architectures for Hexorder

### Option A: Bevy + egui (Current)

- **Form UI**: Adequate but limited for complex editors
- **GPU/Shaders**: Excellent
- **Testing**: No built-in UI test driver; must test at the system/state level
- **Complexity**: Low
- **Maturity**: High

### Option B: Tauri + Bevy Sidecar

- Tauri app (webview frontend) handles all form-based UI
- Bevy compiled as a separate sidecar binary for 3D rendering
- Tauri manages the Bevy process lifecycle (start, stop, bundle)
- Communication via Tauri sidecar IPC (stdin/stdout pipes) or local socket
- **Form UI**: Excellent (full web ecosystem)
- **GPU/Shaders**: Excellent
- **Testing**: Playwright/WebDriver for forms; custom for GPU
- **Complexity**: High (two processes, serialization boundary, state sync)
- **Maturity**: High

### Option C: Iced + wgpu Shader Widget

- Iced (retained-mode Rust GUI framework) handles forms natively
- Custom wgpu rendering in Iced's `Shader` widget for the hex grid
- Single process, shared Rust types, no serialization boundary
- **Form UI**: Good
- **GPU/Shaders**: Good
- **Testing**: Programmatic widget access (Iced is retained-mode with queryable tree)
- **Complexity**: Medium
- **Maturity**: Medium

### Option D: Makepad

- Rust UI framework where all rendering is GPU-accelerated via its own shader DSL
- Forms, text inputs, and 3D content coexist in the same render pipeline
- Closest to the Unity/Unreal "render everything yourself" model
- **Form UI**: Good
- **GPU/Shaders**: Excellent
- **Testing**: Limited (small ecosystem)
- **Complexity**: Medium
- **Maturity**: Low

### Option E: Bevy + Reflect-Driven Form Generation (Unreal-Inspired)

- Keep Bevy + egui as the foundation
- Lean into **bevy-inspector-egui + Bevy's Reflect system** for auto-generated forms
- `#[derive(Reflect, InspectorOptions)]` on game system types → auto-generated editor panels
- Custom egui panels only for specialized editors (hex grid, node graphs)
- Embed **mlua** or **pyo3** for a scripting layer that doubles as user-facing API and test driver
- **Form UI**: Good (auto-generated from types, reduces manual form code significantly)
- **GPU/Shaders**: Excellent
- **Testing**: Embedded scripting layer drives both UI and engine (Autodesk model)
- **Complexity**: Medium
- **Maturity**: Medium-High

### Option F: Qt (CXX-Qt) + Bevy/wgpu (Maya-Inspired)

- Qt Widgets (C++) for the application shell: dockable panels, menus, toolbars, property editors
- CXX-Qt bridges Rust data types into QObjects with `Q_PROPERTY` declarations
- Qt's meta-object system auto-generates property editors from QObject introspection
- Bevy/wgpu renders into a Qt widget via `QVulkanWindow` or raw window handle
- Requires disabling `bevy_winit` and providing a custom window backend
- **Form UI**: Excellent (industry-proven, docking, tree views, data tables, model/view)
- **GPU/Shaders**: Excellent (wgpu renders into Qt-provided surface)
- **Testing**: QTest for UI simulation + introspection; Rust `#[test]` for logic; Squish for
  commercial-grade GUI automation
- **Complexity**: High (bilingual Rust/C++ project, CMake + Cargo build, moc code generation)
- **Maturity**: High (Qt is 30+ years; CXX-Qt is v0.8 heading to 1.0)

---

## Recommendations

### Short-term: Option E (Reflect-driven forms + scripting layer)

This is the most practical path that addresses the form UI pain without leaving the current stack:

1. **Auto-generate forms from Rust types** via bevy-inspector-egui + Reflect, following Unreal's
   UPROPERTY pattern. This eliminates most manual form building for game system data types.
2. **Embed a scripting layer** (Lua via mlua or Python via pyo3) that can drive both the editor UI
   and the simulation engine. This provides testability (Autodesk model) and becomes a user-facing
   feature for scripting game rules.
3. **Keep egui** for custom editors that need specialized layouts (hex grid tools, visual rule
   builders).

### Long-term: Evaluate Option B, C, or F

If form complexity continues to grow beyond what egui + Reflect can handle:

- **Option F (Qt + CXX-Qt + Bevy)** if the priority is Maya-quality editor UI with proven design
  tool patterns (docking, property editors, tree views) and strong testability via QTest. Requires
  comfort with a bilingual Rust/C++ codebase.
- **Option B (Tauri + Bevy sidecar)** if the form UI needs the full web ecosystem (complex data
  tables, rich text editing, drag-and-drop rule builders) and web-based testability (Playwright).
- **Option C (Iced + wgpu)** if staying in a single-process pure-Rust architecture is a priority.

### Anti-recommendation: Building a custom UI framework (Unity/Unreal model)

Both Unity and Unreal invested years and large teams into their custom UI frameworks. Slate and UI
Toolkit exist because of that scale. This path is not viable for Hexorder's team size.

---

## Deep Dive: Qt Framework

### Overview

Qt is a 30+ year old cross-platform C++ application framework (currently Qt 6.10) maintained by The
Qt Company. It is the UI toolkit behind Maya, 3ds Max, Houdini, Substance Painter, VirtualBox, and
many other professional design tools.

Qt provides two UI paradigms:

- **Qt Widgets** — CPU-rendered, native-look desktop widgets. Mature, dense, form-heavy. Includes
  `QDockWidget` (dockable panels), `QTreeView`/`QTableView` (model/view), `QFormLayout` (property
  editors), `QDataWidgetMapper` (form-to-model binding). This is what Autodesk uses.
- **Qt Quick / QML** — a declarative language (JSON-like syntax + JavaScript) with a GPU-accelerated
  scene graph. Better for animated, branded, non-native UIs. More modern but requires more effort
  for traditional design tool patterns (docking, property editors, tree views).

For a design tool like Hexorder, **Qt Widgets is the stronger paradigm** — but it has weaker Rust
bindings (see below).

### Qt's Two UI Paradigms Compared

| Aspect                  | Qt Widgets                                         | Qt Quick / QML                    |
| ----------------------- | -------------------------------------------------- | --------------------------------- |
| Language                | C++                                                | QML + JavaScript (backed by C++)  |
| Rendering               | CPU (QPainter), native look                        | GPU (scene graph), custom look    |
| Animation               | Limited, programmatic                              | First-class, declarative          |
| Native appearance       | Yes (platform-native)                              | No (custom styled)                |
| Property binding        | Manual signal wiring                               | Automatic declarative bindings    |
| Maturity                | 25+ years                                          | ~15 years                         |
| Dock windows            | Built-in (`QDockWidget`)                           | Third-party or custom             |
| Tree/table views        | Mature (`QTreeView`, `QTableView`, model/view)     | Improving (TreeView added Qt 6.4) |
| Form layouts            | Purpose-built (`QFormLayout`, `QDataWidgetMapper`) | Manual                            |
| Design tool suitability | Excellent                                          | Good with extra effort            |

Many modern Qt applications use a **hybrid approach**: Qt Widgets for the application shell (menus,
docks, toolbars) with QML views embedded for specific custom panels.

### Qt's Property System (Meta-Object / Q_PROPERTY)

Qt's meta-object system provides runtime reflection via a compile-time code generator (`moc`). This
is the Qt equivalent of Unreal's UPROPERTY system.

**Declaration:**

```cpp
class UnitType : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString name READ name WRITE setName NOTIFY nameChanged)
    Q_PROPERTY(int hitPoints MEMBER m_hitPoints NOTIFY hitPointsChanged)
    Q_PROPERTY(QColor color READ color WRITE setColor NOTIFY colorChanged DESIGNABLE true)
    // ...
};
```

**Runtime introspection:**

```cpp
const QMetaObject *meta = obj->metaObject();
for (int i = 0; i < meta->propertyCount(); ++i) {
    QMetaProperty prop = meta->property(i);
    QString name = prop.name();        // "hitPoints"
    QVariant value = obj->property(prop.name());  // 100
    QString typeName = prop.typeName(); // "int"
    bool writable = prop.isWritable();  // true
}
```

This enables **auto-generated property editors** — enumerate an object's properties at runtime and
generate the correct widget for each type (spinboxes for ints, color pickers for `QColor`, dropdowns
for enums). Community libraries like Qt-Property-Editor and Qt Designer's built-in property panel
both use this mechanism.

**Comparison to Unreal's UPROPERTY:**

| Feature             | Qt Q_PROPERTY                        | Unreal UPROPERTY                                    |
| ------------------- | ------------------------------------ | --------------------------------------------------- |
| Code generator      | `moc`                                | UHT (Unreal Header Tool)                            |
| Runtime reflection  | `QMetaObject`                        | `UClass` / `UProperty`                              |
| Property editor     | Qt Designer, custom editors          | Details panel                                       |
| Categories/metadata | Limited (`DESIGNABLE`, `SCRIPTABLE`) | Rich (`Category`, `EditAnywhere`, `ClampMin`, etc.) |
| Binding/reactivity  | Signal `NOTIFY` + QML bindings       | Blueprint bindings                                  |

Qt's metadata is less rich than Unreal's (no built-in categories, clamping, or tooltips in the
property macro itself), but can be extended via `Q_CLASSINFO` and custom editor widgets.

### Qt Licensing

Qt offers three license tiers:

**LGPL v3** (primary open-source license):

- Covers the majority of Qt modules (QtCore, QtGui, QtWidgets, QtQml, QtQuick)
- Your application code can be any license if you dynamically link Qt
- Modifications to Qt itself must be shared under LGPL
- Practical for an open-source project — compliance is straightforward

**GPL v3** (specific modules only):

- Required for: Qt Charts, Qt Data Visualization, Qt Quick 3D, Qt Quick Timeline, Qt Virtual
  Keyboard, Qt HTTP Server, and others
- Entire application must be GPL-licensed if these modules are used

**Commercial:**

- Full access to all modules, static linking allowed
- Qt for Small Business: ~530 EUR/year
- Enterprise pricing negotiated

For Hexorder, the LGPL modules cover everything needed for a design tool UI. The GPL-only modules
(Charts, Quick 3D) are not essential.

### Qt + Rust: CXX-Qt

**CXX-Qt** (by KDAB) is the only viable Qt+Rust binding today.

- **Version**: v0.8.0 (December 2024), heading toward 1.0
- **License**: MIT / Apache-2.0
- **Backed by**: KDAB (professional Qt consultancy)
- **Approach**: Built on the `cxx` crate for safe Rust/C++ FFI. Uses a `#[cxx_qt::bridge]` proc
  macro to generate C++ QObject subclasses from Rust structs.

**Example:**

```rust
#[cxx_qt::bridge]
mod ffi {
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, name)]
        #[qproperty(i32, hit_points)]
        #[qproperty(QColor, color)]
        type UnitModel = super::UnitModelRust;
    }
}

#[derive(Default)]
pub struct UnitModelRust {
    name: QString,
    hit_points: i32,
    color: QColor,
}
```

This generates a real C++ QObject with proper `Q_PROPERTY` declarations. The Rust struct owns the
data; Qt sees a standard QObject that works with property editors, QML bindings, and `QMetaObject`
introspection.

**What CXX-Qt covers well:**

- QObject subclasses with properties, signals, slots
- QML integration (`#[qml_element]`, `#[qml_singleton]`)
- Common Qt types via `cxx-qt-lib` (QString, QColor, QVariant, QList, QHash, QMap, etc.)
- Cross-thread signal/slot communication
- Multi-crate projects

**What CXX-Qt does NOT cover:**

- **No Rust-native QWidget wrappers** — cannot write `QDockWidget`, `QTreeView`, `QFormLayout` code
  in Rust. Widget-level UI must be written in C++.
- Not all Qt types are wrapped in `cxx-qt-lib`
- Still marked "early development" (though the bridge macro is stable since v0.7)

**Other Qt+Rust projects (not recommended):**

- **qmetaobject-rs** — passively maintained, author shifted focus to Slint
- **ritual / rust-qt** — effectively dormant, no Qt 6 support

### Qt + GPU Rendering Integration

Qt provides several mechanisms for embedding custom GPU rendering inside the UI:

**QVulkanWindow:**

- Sets up a Vulkan instance, surface, and swap chain
- Provides `VkSurfaceKHR` that can be passed to wgpu
- Embeddable in a QWidget layout via `QWidget::createWindowContainer()`

**QWidget::winId() (raw window handle):**

- Returns the platform-native window handle (NSView on macOS, HWND on Windows)
- Can be used with `raw-window-handle` to create a wgpu surface
- Bypasses Qt's rendering stack — wgpu owns the surface entirely

**QRhi (Qt Rendering Hardware Interface):**

- Qt 6's abstraction over Vulkan, Metal, Direct3D 12, and OpenGL ES
- `QSGRenderNode` allows injecting custom rendering into the Qt Quick scene graph
- Increasingly exposed but still partially private API

**Render-to-texture:**

- Render with wgpu to an offscreen texture, blit into a Qt widget or QML item
- Higher latency but simpler integration, no surface ownership conflicts

**Embedding Bevy specifically** is harder because Bevy assumes ownership of the window and render
loop via `bevy_winit`. You would need to disable `bevy_winit` and provide a custom window backend
that wraps a Qt widget's native handle. The Bevy community has discussed this approach but no
production-ready solution exists.

### Qt Testing Infrastructure

**QTest** (built-in):

- GUI event simulation: `QTest::mouseClick()`, `QTest::keyClick()`, `QTest::mousePress()`,
  `QTest::mouseMove()` — simulate real user interactions on any QWidget
- Events go through Qt's event system, triggering the same signal/slot chains as real input
- Widget introspection: `QObject::findChildren<T>()` to find widgets by type or name
- Property introspection via `QMetaObject` — enumerate and verify all property values at runtime
- `QApplication::allWidgets()` — get all active widgets in the application
- Data-driven tests via `_data()` functions
- Command-line and Qt Creator integration

**Squish** (commercial, now owned by The Qt Company):

- Record-and-playback GUI test automation
- Object-map based widget identification (survives layout changes)
- Scripting in Python, JavaScript, Ruby, Perl, Tcl
- The most comprehensive Qt GUI testing tool available

**From Rust:**

- Business logic tested with standard Rust `#[test]`
- Qt UI interaction tested with C++ QTest harnesses
- CXX-Qt properties are visible to QTest via `QMetaObject` introspection
- No Rust-native way to call QTest APIs currently

### Practical Architecture for Hexorder with Qt

```
┌─────────────────────────────────────────────┐
│  Qt Widgets (C++)                           │
│  ┌──────────┐ ┌──────────┐ ┌─────────────┐ │
│  │ Dock:    │ │ Dock:    │ │ Dock:       │ │
│  │ Ontology │ │ Property │ │ Rules       │ │
│  │ Tree     │ │ Editor   │ │ Editor      │ │
│  │ (QTree)  │ │ (QForm)  │ │ (QForm)     │ │
│  └──────────┘ └──────────┘ └─────────────┘ │
│  ┌────────────────────────────────────────┐ │
│  │  3D Viewport (QWidget + wgpu surface)  │ │
│  │  ← Bevy renderer draws here            │ │
│  └────────────────────────────────────────┘ │
├─────────────────────────────────────────────┤
│  CXX-Qt Bridge                              │
├─────────────────────────────────────────────┤
│  Rust (Bevy ECS, game logic, simulation)    │
└─────────────────────────────────────────────┘
```

- C++ owns the UI shell (docking, menus, toolbars, form panels)
- Rust owns all game logic, simulation, and rendering via Bevy/wgpu
- CXX-Qt bridges Rust data types into QObjects so property editors auto-generate from them
- QTest tests the UI; Rust `#[test]` tests the logic

### Qt Assessment for Hexorder

| Dimension                 | Rating    | Notes                                                           |
| ------------------------- | --------- | --------------------------------------------------------------- |
| Form UI quality           | Excellent | Industry-proven for design tools (Maya, Houdini, Substance)     |
| GPU rendering integration | Feasible  | QVulkanWindow + raw-window-handle; significant integration work |
| Rust ergonomics           | Mixed     | QML bindings good via CXX-Qt; QWidget code stays in C++         |
| Testing                   | Strong    | QTest + Squish cover UI; Rust covers logic                      |
| Build complexity          | High      | moc, CMake, C++ compilation alongside Cargo                     |
| Team skill requirements   | High      | Must be comfortable in both Rust and C++                        |
| Licensing                 | Good      | LGPL covers all needed modules for open-source project          |
| Maturity                  | Excellent | 30+ years, massive ecosystem                                    |

**Biggest trade-off:** Hexorder becomes a **bilingual project** — Rust for the engine, C++ for the
UI layer, with CXX-Qt as the bridge. That is the cost of getting Maya-quality editor UI.

---

## Deep Dive: Rust UI Building Blocks (Composable Stack)

### Motivation

Every option surveyed so far involves a trade-off: adopt a monolithic framework and accept its
limitations, or cross into C++/web territory for richer UI. There is a third path: **compose a
custom stack from proven, standalone Rust crates** — building only the domain-specific widget layer
while getting windowing, rendering, layout, text, accessibility, and testing for free.

This is conceptually what Blender did (custom UI from lower-level primitives), but using existing
Rust building blocks rather than writing everything from scratch in C.

### The Crate Ecosystem (2025-2026)

#### GPU 2D Rendering: Vello

**Vello** (Linebender project) is a GPU compute-centric 2D vector renderer built on wgpu.

- **Status**: Alpha (v0.6 on crates.io, active development well past that)
- **Architecture**: Three rendering backends — GPU compute shaders, CPU (sparse-strip + Fearless
  SIMD, "likely the fastest CPU-only renderer in Rust"), and Hybrid (CPU preprocessing + GPU
  rasterization, 30% improvement in overdraw handling as of December 2025)
- **Capabilities**: Paths, shapes, gradients, text (via Parley integration), compositing
- **Standalone**: Yes — not tied to any framework
- **Users**: Xilem/Masonry (rendering backend), bevy_vello, Floem (optional backend)
- **Risk**: API churn expected before 1.0

#### Layout Engine: Taffy

**Taffy** (maintained by Dioxus Labs) implements CSS Block, Flexbox, and CSS Grid layout.

- **Status**: v0.7.2 (January 2025) — stable, pre-1.0 but mature
- **Architecture**: Pure layout math. Generic over node storage — you provide the tree, Taffy
  computes positions and sizes. No rendering, no DOM.
- **Users**: Bevy (`bevy_ui`), Dioxus, Xilem/Masonry, Blitz (Servo-derived), Zed editor (fork)
- **Standalone**: Yes — completely framework-agnostic
- **Coverage**: Flexbox + CSS Grid cover most form layout needs. Docking and split panes are
  widget-level concerns built on top of Taffy's layout primitives, not provided by Taffy directly.
- **Risk**: Low — broad production adoption

#### Text Rendering: Parley + Fontique

**Parley** (Linebender) handles rich text layout — line breaking, glyph positioning, bidirectional
text, text selection.

- **Status**: v0.3.0 (February 2025) — active, rapid development
- **Architecture**: Uses Fontique for font discovery/fallback, HarfRust for shaping, ICU4X for
  internationalization. Designed to be framework-agnostic.
- **Standalone**: Yes
- **Risk**: Medium — API unstable, active churn

**Alternative: cosmic-text** (System76) — the most battle-tested option, shipping in the COSMIC
desktop environment (Pop!\_OS 24.04 LTS). More monolithic (shaping + layout + rasterization +
editing in one crate) but proven at scale. If Linebender stack alignment isn't a priority,
cosmic-text is the safer choice.

#### Text-to-Screen: Glyphon

**Glyphon** — simple wgpu text rendering middleware. Uses cosmic-text for shaping/layout and etagere
for atlas packing. Renders text into an existing wgpu render pass.

- **Status**: v0.10.0 (December 2025) — actively maintained
- **Standalone**: Yes — works with any wgpu application
- **Use case**: The "last mile" renderer for getting text onto the screen. Good for overlaying text
  on a 3D viewport.

#### Windowing: winit

**winit** is the de facto standard for cross-platform window creation and event loops in Rust.

- **Status**: v0.30.12 (stable). v0.31 in beta.
- **Users**: Nearly everything — wgpu, egui, Bevy, Xilem/Masonry, Iced, Floem
- **Standalone**: Yes
- **Risk**: Low — universal adoption

Note: Glazier (Linebender's former alternative) is **deprecated** — the team migrated to winit and
contributed improvements upstream.

#### Accessibility: AccessKit

**AccessKit** exposes a framework-agnostic accessibility tree that maps to platform-native a11y APIs
(UIA on Windows, NSAccessibility on macOS, AT-SPI on Linux).

- **Status**: Stable — platform adapters for Windows, macOS, and Linux (GNOME)
- **Architecture**: UI toolkits push tree updates to AccessKit, which translates them into
  platform-native APIs. The tree schema is the universal contract.
- **Users**: egui (default integration), Xilem/Masonry, planned for Vizia
- **Standalone**: Yes — designed to be framework-agnostic

#### Testing via Accessibility: kittest

**kittest** (by Rerun) wraps AccessKit's consumer tree to provide a **Testing Library-style API**
for GUI testing in Rust.

- Query UI elements by role, label, or properties
- Simulate clicks, keypresses, focus changes
- Assert on widget state
- egui_kittest builds on this for egui-specific testing with screenshot comparison

**This is the key discovery.** The accessibility tree serves the same structural role as the DOM
does for WebDriver — a framework-agnostic, queryable tree of UI elements. Building with AccessKit
integration gives you **both accessibility compliance and automated testability from the same
investment**. No external injection tools (AltTester), no separate test framework, no web-only
limitation.

#### Scripting: mlua (LuaJIT)

**mlua** is the standard crate for embedding Lua in Rust.

- **Status**: v0.11.5 (January 2026) — stable, actively maintained
- **Supports**: Lua 5.1–5.5, LuaJIT, Luau (Roblox's typed Lua)
- **Features**: Async/await, serde integration, sandboxing, UserData trait for exposing Rust types
- **LuaJIT performance**: 10–100x faster than Rhai for compute-bound work

**Alternatives:**

- **Rhai** (v1.24) — pure Rust scripting language, no native dependency, but significantly slower.
  Good for safe execution of untrusted config scripts. Not suitable for simulation-heavy workloads.
- **PyO3** (v0.28) — Python embedding. Heavy runtime (~30MB+), GIL overhead. Best if users need
  Python's data science ecosystem. Overkill for game rule scripting.

Lua is the industry standard for game tool scripting. For Hexorder, mlua with LuaJIT is the
strongest choice — it becomes a user-facing feature (scriptable game rules, batch automation) and an
integration test driver alongside kittest.

#### Reflection: bevy_reflect (standalone)

**bevy_reflect** can be used **outside of Bevy** — it is published as a standalone crate and does
not require the ECS, renderer, or any Bevy runtime.

- **Status**: v0.18.0 — stable, actively maintained
- **Capabilities**: Derive `Reflect` on structs/enums/tuple structs, runtime type introspection,
  `TypeRegistry` for metadata, serialization without serde
- **Dependencies**: Pulls in lightweight Bevy utility crates (`bevy_utils`, `bevy_ptr`), but not the
  engine itself
- **Pattern**: bevy-inspector-egui demonstrates auto-generating UI from `Reflect` types — the same
  pattern works with any widget system, not just egui
- **Alternatives**: None — there are no comparable standalone Rust reflection crates. Rust language
  compile-time reflection is under RFC discussion but not available yet.

### The Linebender Stack

Many of these crates come from the **Linebender** organization, which maintains them as a coherent
set of composable layers:

```
┌─────────────────────────────────────────────────────────┐
│  Xilem (declarative view framework)          — Alpha    │
├─────────────────────────────────────────────────────────┤
│  Masonry (retained widget tree)              — Alpha    │
├─────────────────────────────────────────────────────────┤
│  Vello        Taffy       Parley      AccessKit         │
│  (2D render)  (layout)    (text)      (a11y)            │
│  Alpha        Stable      Alpha       Stable            │
├─────────────────────────────────────────────────────────┤
│  wgpu         winit       Fontique    Kurbo   Peniko    │
│  (GPU)        (window)    (fonts)     (geom)  (color)   │
│  Stable       Stable      Alpha       Stable  Active    │
└─────────────────────────────────────────────────────────┘
```

**Key insight**: You do not need Xilem or Masonry. The lower and middle layers (Vello, Taffy,
Parley, AccessKit, winit, wgpu) are usable independently. You can build a **custom widget layer
directly on top of these** — tailored to design tool needs rather than general-purpose UI.

### Other Rust UI Frameworks Evaluated

**Floem** (Lapce project):

- Reactive UI library, used in production by Lapce (Rust IDE)
- Signal-based fine-grained reactivity, wgpu rendering (vello or vger backends)
- **Disqualified**: Poor accessibility — Windows Narrator cannot see into Floem windows. IME support
  broken. Accessibility failures make it unsuitable for a testability-focused architecture.

**Slint** (SixtyFPS GmbH):

- Production-ready (v1.15), commercial backing, own DSL (`.slint` files)
- **Disqualified**: Monolithic — cannot swap in Vello or Taffy. GPL license for open-source use. Not
  composable.

**COSMIC Toolkit** (System76):

- Built on a heavily modified iced fork. Ships with Pop!\_OS 24.04 LTS.
- **Disqualified**: Tightly coupled to COSMIC's design language and iced fork. Poor accessibility
  (iced limitation). Framework, not toolkit of building blocks.

**GPUI** (Zed Industries):

- GPU-accelerated hybrid immediate/retained-mode UI framework. Powers the Zed code editor.
- Apache 2.0, published as standalone crate (`gpui` 0.2.2). Uses Taffy for layout, Metal (macOS) /
  wgpu (Linux/Windows) for rendering, cosmic-text for text. Targets 120 FPS.
- **gpui-component** (by Longbridge, 10.3k stars) provides 60+ production-grade widgets: text inputs
  with validation, buttons, virtualized tables/lists, split panes, markdown/HTML rendering, charts,
  and a full code editor component. Used in production by Longbridge Pro (financial app).
- Several real apps built outside Zed: Loungy (launcher), vleer (music), zedis (Redis GUI),
  gpui-ghostty (terminal emulator), Longbridge Pro.
- Zed migrated from blade to **wgpu** in early 2026, meaning GPUI and Bevy now share the same GPU
  abstraction. Zed team noted this opens "future possibilities for custom shaders and embedding into
  other wgpu apps, like Bevy."
- **Disqualified for testability**: No accessibility support — "built from scratch without native
  accessibility features," acknowledged as extending "beyond version 1.0." No AccessKit integration.
  No queryable widget tree for automated UI testing. Screen reader support "remains essentially
  non-functional." This directly conflicts with the testability requirement — the 60+ components
  cannot be driven by kittest or any accessibility-based test framework.
- Additional concerns: Pre-1.0 with frequent breaking changes. Documentation is thin ("read the Zed
  source code"). Zed-centric development — not actively supported as a standalone framework. No
  documented path for embedding a 3D/Bevy viewport (though wgpu sharing makes it theoretically
  feasible).

### Option G: Composable Rust Stack (Proposed)

Assemble a custom editor UI from proven lower-layer crates:

```
┌──────────────────────────────────────────────────────────────┐
│  Custom Widget Layer (Hexorder-specific)                     │
│  ┌──────────┐ ┌──────────┐ ┌────────┐ ┌──────────────────┐  │
│  │ Dock     │ │ Property │ │ Tree   │ │ Form fields:     │  │
│  │ Panels   │ │ Editor   │ │ View   │ │ text, dropdown,  │  │
│  │          │ │ (auto-   │ │        │ │ slider, color,   │  │
│  │          │ │ generated│ │        │ │ checkbox         │  │
│  │          │ │ via      │ │        │ │                  │  │
│  │          │ │ Reflect) │ │        │ │                  │  │
│  └──────────┘ └──────────┘ └────────┘ └──────────────────┘  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  3D Viewport (wgpu / Bevy renderer)                    │  │
│  └────────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────┤
│  Vello (2D)  │ Taffy (layout) │ Parley (text) │ AccessKit   │
├──────────────────────────────────────────────────────────────┤
│  wgpu        │ winit          │ bevy_reflect  │ mlua (Lua)  │
├──────────────────────────────────────────────────────────────┤
│  Bevy ECS    │ Game logic     │ Simulation    │ Contracts   │
└──────────────────────────────────────────────────────────────┘
```

**What you get for free (existing crates):**

| Concern                                     | Crate                   | Maturity                |
| ------------------------------------------- | ----------------------- | ----------------------- |
| Window management, platform events          | winit                   | Stable                  |
| GPU 2D rendering (shapes, paths, text)      | Vello                   | Alpha but functional    |
| Layout computation (flexbox, CSS grid)      | Taffy                   | Stable                  |
| Text shaping, layout, line breaking         | Parley (or cosmic-text) | Alpha (or stable)       |
| Accessibility tree                          | AccessKit               | Stable                  |
| Automated UI test driver                    | kittest                 | Stable                  |
| Type introspection for auto-generated forms | bevy_reflect            | Stable                  |
| Scripting runtime                           | mlua (LuaJIT)           | Stable                  |
| 3D rendering                                | wgpu / Bevy             | Stable (already exists) |

**What you build:**

- **~10–15 widget types**: text field, dropdown/select, slider, color picker, checkbox, radio, tree
  view, tab bar, dock panel, split pane, scroll region, toolbar, menu bar
- **Property editor generator**: connects bevy_reflect introspection to the widget system — given a
  `Reflect` type, auto-generate the appropriate form fields (the Unreal UPROPERTY → Details Panel
  equivalent)
- **Docking system**: split panes, resizable panels, tab groups — built on top of Taffy layout
  primitives
- **Integration layer**: 2D form UI and 3D viewport coexisting in the same winit window, sharing
  wgpu

**Testing architecture:**

```
┌─────────────────────────────────────────────────┐
│  Test Levels                                     │
├──────────┬──────────────────┬───────────────────┤
│ Unit     │ Integration      │ UI Interaction     │
│ (Rust    │ (kittest +       │ (mlua scripts +    │
│ #[test]) │ AccessKit tree)  │ kittest queries)   │
│          │                  │                    │
│ Logic,   │ Query by role/   │ Full scenarios:    │
│ ECS,     │ label, simulate  │ "create unit type, │
│ contracts│ clicks/keys,     │ set properties,    │
│          │ assert state     │ run simulation,    │
│          │                  │ verify results"    │
└──────────┴──────────────────┴───────────────────┘
```

- **Unit tests**: Standard Rust `#[test]` for logic, ECS systems, contracts
- **UI interaction tests**: kittest queries the AccessKit tree by role/label, simulates clicks and
  keypresses, asserts widget state — same pattern as Testing Library for web
- **Integration/scenario tests**: mlua scripts drive full workflows (create entities, set
  properties, run simulations, verify outcomes) — the Maya/Blender scripting-as-test-driver model
- **Screenshot tests**: Vello can render to an offscreen surface for pixel-level regression testing

### Option G Assessment

| Dimension        | Rating                  | Notes                                                |
| ---------------- | ----------------------- | ---------------------------------------------------- |
| Form UI quality  | You control the ceiling | Build exactly the widgets a design tool needs        |
| GPU rendering    | Excellent               | Vello (2D) + wgpu/Bevy (3D) in same pipeline         |
| Testability      | Excellent               | AccessKit + kittest (structural) + mlua (scripting)  |
| Language         | Pure Rust               | No C++, no JavaScript, all Cargo                     |
| Build complexity | Low-Medium              | All Cargo dependencies, no moc/CMake                 |
| Widget maturity  | You build this          | ~10–15 widgets, bounded problem                      |
| Reflection → UI  | bevy_reflect + custom   | Same pattern as bevy-inspector-egui                  |
| Scripting        | mlua (LuaJIT)           | User-facing game rules + test automation             |
| Risk             | Medium                  | Lower layers proven; widget layer is your investment |

### Option G vs Other Options

| Concern             | egui (A)      | Qt (F)             | Composable Stack (G)       |
| ------------------- | ------------- | ------------------ | -------------------------- |
| Form UI quality     | Adequate      | Excellent          | You control the ceiling    |
| GPU rendering       | Via Bevy      | Needs C++ bridge   | Native (Vello + wgpu)      |
| Testability         | Weak          | QTest + Squish     | AccessKit + kittest + mlua |
| Language            | Pure Rust     | Rust + C++         | Pure Rust                  |
| Build complexity    | Low           | High (CMake + moc) | Low-Medium (all Cargo)     |
| Widget maturity     | Mature (egui) | Decades of polish  | You build ~10–15 widgets   |
| Time to first panel | Immediate     | Medium (C++ setup) | Medium (widget layer)      |

### Risk Assessment

**What could go wrong:**

1. **Vello API churn** — alpha status means breaking changes. Mitigation: wrap Vello behind a thin
   rendering trait so the widget layer is not coupled to Vello's API directly.
2. **Parley API churn** — same as Vello. Mitigation: cosmic-text as a stable fallback.
3. **Widget layer scope creep** — the "just build 10 widgets" plan grows into a general-purpose UI
   toolkit. Mitigation: strict scope to design tool needs. No animations, no mobile, no theming
   beyond light/dark.
4. **AccessKit coverage gaps** — if a widget type doesn't map cleanly to a11y roles, kittest queries
   become awkward. Mitigation: design widgets with a11y roles in mind from the start.
5. **No existing reference implementation** — nobody has shipped a production design tool on this
   exact stack. You are the pioneer.

**What de-risks it:**

1. The lower layers (Taffy, AccessKit, winit, wgpu, mlua, bevy_reflect) are all individually stable
   and production-proven.
2. The widget layer is a bounded problem — design tools need a known set of widgets, not infinite
   generality.
3. bevy_reflect handles the hardest part (runtime introspection for auto-generated forms) — you
   write the mapping, not the reflection system.
4. kittest provides the test driver protocol — you don't need to invent testing infrastructure.
5. The scripting layer (mlua) is both a product feature and a test tool — dual investment.

---

## Key Takeaways

1. **Every major design tool solves the form + GPU viewport problem in one process.** The sidecar /
   two-process approach (Option B) is architecturally novel but unproven at this scale.

2. **Reflection-driven form generation is the highest-leverage pattern.** Unreal's UPROPERTY system
   is what makes their editor scale — not Slate itself. Bevy's Reflect system offers the same
   capability and works standalone.

3. **Embedded scripting is the industry-standard test driver for native design tools.** Maya,
   Blender, Houdini, and (partially) Unreal all use embedded Python as both user API and test
   driver. Web-based tools get Playwright. There is no good middle ground for native apps without a
   scripting layer.

4. **The web UI path (Tauri) trades one problem for another.** You gain form UI quality and
   testability but take on IPC complexity, state synchronization, and a dual-technology stack.

5. **AccessKit + kittest is the Rust-native answer to UI testability.** The accessibility tree
   provides the same queryable structure as the DOM, enabling Testing Library-style UI tests without
   a web runtime or external injection tools. This should be a non-negotiable dependency regardless
   of which UI path is chosen.

6. **A composable Rust stack (Option G) is viable.** The lower layers (winit, wgpu, Taffy,
   AccessKit, bevy_reflect, mlua) are individually stable. The investment is in the widget layer
   (~10–15 domain-specific widgets), which is a bounded problem for a design tool. The risk is being
   a pioneer — no production design tool has shipped on this exact combination yet.

7. **Qt (Option F) remains the proven industrial choice** if you accept the bilingual (Rust + C++)
   trade-off. CXX-Qt bridges the data layer cleanly, but the UI shell stays in C++.

8. **GPUI has the richest component ecosystem in Rust** (60+ production widgets via gpui-component),
   but the complete absence of accessibility/AccessKit makes it untestable via kittest. The wgpu
   migration in 2026 makes Bevy integration theoretically feasible, but no documented path exists.

9. **Dioxus Native (Blitz) is heading toward the ideal convergence** — HTML/CSS form controls,
   GPU-rendered via wgpu, AccessKit integration, shared lower crates with Bevy. It is pre-alpha
   today but represents the most promising future target.

---

## Strategy

### The optimal framework to build from today does not exist yet.

Every option requires a trade-off:

| Framework             | Form Controls    | Testable         | GPU + Bevy   | Pure Rust           | Production-Ready   |
| --------------------- | ---------------- | ---------------- | ------------ | ------------------- | ------------------ |
| egui + egui_kittest   | Good             | Yes              | Yes          | Yes                 | **Yes**            |
| GPUI + gpui-component | Excellent        | **No**           | Possible     | Yes                 | Pre-1.0            |
| Dioxus Native (Blitz) | Excellent (HTML) | Planned          | Shared wgpu  | Yes                 | **No (pre-alpha)** |
| Dioxus webview        | Excellent (HTML) | Yes (Playwright) | Sidecar only | Yes                 | Yes                |
| Qt (CXX-Qt)           | Excellent        | Yes (QTest)      | Feasible     | **No (Rust + C++)** | Yes                |
| Slint                 | Good             | Partial          | Separate     | Yes (GPL)           | Yes                |
| Composable Stack (G)  | Build your own   | Yes              | Yes          | Yes                 | Lower layers only  |

Only egui checks every box today. Its ceiling is form quality. But the floor is: it ships, it tests,
it runs.

### Phase 1: Build on egui + egui_kittest + bevy_reflect (now)

**Add testability and reflection-driven forms to the existing stack.**

Actions:

1. Add **egui_kittest** — enables AccessKit-based UI interaction testing (query by role/label,
   simulate clicks/keypresses, assert widget state)
2. Derive **`Reflect`** on all game system data types (unit types, terrain types, properties, rules)
3. Use **bevy-inspector-egui** patterns to auto-generate property editor panels from reflected types
   — the Unreal UPROPERTY → Details Panel equivalent, eliminating manual form building for most data
   types
4. Add **mlua** (LuaJIT) as an embedded scripting layer — serves dual purpose as user-facing game
   rule scripting and integration-level test driver (the Maya/Blender model)
5. Keep custom egui panels only for specialized editors that need bespoke layouts (hex grid tools,
   visual rule builders)

Testing architecture:

```
Unit tests          → Rust #[test] — logic, ECS systems, contracts
UI interaction tests → egui_kittest — query AccessKit tree, simulate clicks, assert state
Integration tests   → mlua scripts — full workflows (create entity, set properties, run sim, verify)
Screenshot tests    → egui_kittest snapshot — pixel-level regression for UI panels
```

**Design principle: keep the data layer UI-framework-agnostic.** Game system types derive `Reflect`
and live in contracts. The UI layer reads reflected types and generates forms. If the UI framework
changes later, the data layer does not.

### Phase 2: Evaluate Dioxus Native (Blitz) when it reaches beta

**Monitor Blitz maturity. When it stabilizes, evaluate as a migration target.**

Blitz delivers the combination this research identified as optimal:

- **HTML/CSS** as the form description language (the richest, most well-understood form model)
- **GPU-rendered** via Vello + wgpu (no webview, no browser engine)
- **AccessKit** for testability (same kittest approach as Phase 1)
- **Shared lower crates with Bevy**: Taffy, Parley, AccessKit, winit, wgpu

Watch for:

- Blitz reaching beta (targeted 2026) with stable form control support (`<input>`, `<select>`,
  `<textarea>`, scroll regions)
- AccessKit integration landing and working with kittest
- CSS support covering design tool needs (grid layout, overflow, position)
- A working `blitz-in-bevy` integration pattern (render Blitz to wgpu texture, composite into Bevy)

The migration from Phase 1 to Phase 2 would be a **panel rewrite, not an architecture rewrite**:

- ECS, simulation, contracts, scripting layer: unchanged
- Data types with `Reflect`: unchanged (Blitz forms would read the same reflected types)
- Test infrastructure: kittest still works (same AccessKit tree, different renderer)
- Only the UI panels change: egui code → RSX/HTML templates

### Phase 3: Long-term ecosystem convergence

**Bevy and Dioxus/Blitz are converging on the same lower crates.** As Bevy migrates from cosmic-text
to Parley (all 3 subtasks completed), the shared stack becomes: Taffy + Parley + AccessKit + winit +
wgpu. This convergence makes deeper integration increasingly natural — potentially a first-class
Blitz-in-Bevy plugin rather than a texture-blit workaround.

Also watch:

- **Woodpecker UI** — a Bevy ECS-driven UI using Vello + Taffy + Parley (same rendering stack as
  Blitz, but Bevy-native). Could emerge as the Bevy-first answer to this problem.
- **Bevy's own UI evolution** — BSN (Bevy Scene Notation) and the Feathers widget library. If Bevy
  ships a capable retained-mode UI with AccessKit integration, the need for an external framework
  diminishes.
- **GPUI accessibility** — if Zed adds AccessKit support, GPUI + gpui-component immediately becomes
  the strongest option (60+ tested widgets, shared wgpu, Apache 2.0).

### Summary

```
Today                          Near-term                       Target
─────                          ─────────                       ──────
egui + egui_kittest     →      + bevy_reflect auto-       →    Dioxus Native (Blitz)
(testable forms,               generated property editors      (HTML/CSS forms,
 already integrated)           (Unreal UPROPERTY pattern)      GPU-rendered via wgpu,
                                                               AccessKit testable,
+ mlua for scripting    →      + mlua integration tests   →    shared crate stack
                                                               with Bevy)

Data layer (Reflect + contracts) stays constant across all phases.
```

The investment in `Reflect`-based data types, AccessKit-driven testing, and mlua scripting pays off
regardless of which UI framework renders the panels. Build the right abstractions now; swap the
renderer later.
