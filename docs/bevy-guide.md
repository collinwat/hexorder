# Bevy 0.18 Developer Guide for Hexorder

> Canonical reference for Bevy 0.18 patterns, conventions, and pitfalls. Updated: 2026-02-08 | Bevy
> 0.18 (released 2026-01-13)

---

## Table of Contents

1. [Quick Reference](#1-quick-reference)
2. [ECS Fundamentals](#2-ecs-fundamentals)
3. [Systems and Scheduling](#3-systems-and-scheduling)
4. [Messages and Events](#4-messages-and-events)
5. [Queries](#5-queries)
6. [Plugin Architecture](#6-plugin-architecture)
7. [State Management](#7-state-management)
8. [3D Rendering](#8-3d-rendering)
9. [Input Handling](#9-input-handling)
10. [Asset System](#10-asset-system)
11. [Transforms and Hierarchy](#11-transforms-and-hierarchy)
12. [Picking (Pointer Interaction)](#12-picking)
13. [Testing Patterns](#13-testing-patterns)
14. [Hexorder-Specific Conventions](#14-hexorder-specific-conventions)
15. [Bevy 0.18 Migration Notes](#15-bevy-018-migration-notes)
16. [hexx 0.22 Reference](#16-hexx-022-reference)
17. [macOS Platform](#17-macos-platform)
18. [Common Pitfalls](#18-common-pitfalls)
19. [GPU Pipeline Startup and Window Flash Prevention](#19-gpu-pipeline-startup-and-window-flash-prevention)

---

## 1. Quick Reference

### Derive Macros

| Derive      | Required Co-derives                          | Purpose                                        |
| ----------- | -------------------------------------------- | ---------------------------------------------- |
| `Component` | `Debug` (project rule)                       | Per-entity data                                |
| `Resource`  | `Debug` (project rule)                       | Global singleton data                          |
| `Event`     | `Debug` (project rule)                       | Observer/trigger events (immediate)            |
| `Message`   | `Debug` (project rule)                       | Buffered pull-based messages (double-buffered) |
| `States`    | `Debug, Clone, PartialEq, Eq, Hash, Default` | Finite state machine                           |
| `SystemSet` | `Debug, Clone, PartialEq, Eq, Hash`          | System ordering groups                         |

### System Parameter Types

| Parameter          | Purpose                                               |
| ------------------ | ----------------------------------------------------- |
| `Res<T>`           | Read-only resource                                    |
| `ResMut<T>`        | Mutable resource                                      |
| `Query<D, F>`      | Entity component access (D = data, F = filter)        |
| `Commands`         | Deferred spawn/despawn/insert/remove                  |
| `MessageReader<M>` | Read buffered messages                                |
| `MessageWriter<M>` | Write buffered messages                               |
| `Local<T>`         | Per-system persistent local state                     |
| `Option<Res<T>>`   | Optional resource (may not exist)                     |
| `Single<D, F>`     | Exactly one matching entity (skips system if 0 or 2+) |

Max 16 parameters per system function.

### Schedule Labels (execution order per frame)

| Schedule           | Purpose                              |
| ------------------ | ------------------------------------ |
| `Startup`          | Runs once before first Update        |
| `First`            | Start of every frame                 |
| `PreUpdate`        | Engine internals before user logic   |
| `StateTransition`  | Pending state transitions            |
| `RunFixedMainLoop` | Runs FixedUpdate N times             |
| `Update`           | **Main game logic**                  |
| `PostUpdate`       | Engine internals after user logic    |
| `Last`             | End of every frame                   |
| `FixedUpdate`      | Fixed timestep (physics, simulation) |
| `OnEnter(S)`       | When entering state S                |
| `OnExit(S)`        | When exiting state S                 |

---

## 2. ECS Fundamentals

### Components

Data-only structs attached to entities. No logic — logic lives in systems.

```rust
// Marker component (no data)
#[derive(Component, Debug)]
pub struct HexTile;

// Data component
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexPosition {
    pub q: i32,
    pub r: i32,
}
```

**Required Components** (auto-insert dependencies):

```rust
#[derive(Component)]
#[require(Transform, Visibility)]
struct MyGameObject;
```

Inserting `MyGameObject` automatically inserts `Transform` and `Visibility` if not already present
(uses `Default`).

### Resources

Global singletons. One instance per type in the entire World.

```rust
// With manual Default
#[derive(Resource, Debug)]
pub struct CameraState {
    pub target_position: Vec2,
    pub target_scale: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            target_position: Vec2::ZERO,
            target_scale: 10.0,
        }
    }
}

// With derive Default
#[derive(Resource, Debug, Default)]
pub struct SelectedHex {
    pub position: Option<HexPosition>,
}
```

**Registration:**

```rust
app.init_resource::<CameraState>();           // uses Default
app.insert_resource(HexGridConfig { ... });   // explicit value
commands.insert_resource(MyResource { ... }); // from a system
```

### Entity Spawning

Entities are spawned with component tuples (bundles are deprecated — use tuples):

```rust
fn setup(mut commands: Commands) {
    // Spawn with component tuple
    let entity = commands.spawn((
        HexTile,
        HexPosition::new(0, 0),
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(material_handle.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();

    // Modify existing entity
    commands.entity(entity).insert(Terrain { terrain_type: TerrainType::Forest });
    commands.entity(entity).remove::<Terrain>();

    // Despawn
    commands.entity(entity).despawn();
}
```

Commands are **deferred** — structural changes apply at synchronization points between systems.

---

## 3. Systems and Scheduling

### System Signatures

Systems are plain functions with injectable parameters:

```rust
// Read-only
pub fn print_positions(query: Query<&HexPosition>) {
    for pos in &query {
        println!("({}, {})", pos.q, pos.r);
    }
}

// Mutable resource + query
pub fn smooth_camera(
    time: Res<Time>,
    mut camera_state: ResMut<CameraState>,
    mut query: Query<(&mut Transform, &mut Projection), With<TopDownCamera>>,
) {
    let Ok((mut transform, mut projection)) = query.single_mut() else {
        return;
    };
    // ...
}
```

### Ordering

**Chaining** (sequential execution within a tuple):

```rust
app.add_systems(
    Startup,
    (setup_grid_config, setup_materials, spawn_grid).chain(),
);
```

**System Sets** (named groups with ordering):

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum CameraSet {
    Input,
    Apply,
}

app.configure_sets(Update, CameraSet::Apply.after(CameraSet::Input))
    .add_systems(Update, (
        keyboard_pan.in_set(CameraSet::Input),
        mouse_pan.in_set(CameraSet::Input),
        scroll_zoom.in_set(CameraSet::Input),
        smooth_camera.in_set(CameraSet::Apply),
    ));
```

**Run Conditions:**

```rust
app.add_systems(Update, game_logic.run_if(in_state(GameState::Playing)));
```

### Important: `.after(fn)` Does Not Work on Bare Functions

In Bevy 0.18, bare function items no longer implement `IntoSystemSet`. You cannot write
`.after(my_system_fn)` on a system in a tuple. Use `.chain()` or explicit `SystemSet` types instead.

---

## 4. Messages and Events

### Critical Terminology Change (Bevy 0.17+)

Bevy 0.17 renamed the buffered event system:

| Old (pre-0.17)         | New (0.17+)              | Purpose                         |
| ---------------------- | ------------------------ | ------------------------------- |
| `#[derive(Event)]`     | `#[derive(Message)]`     | Buffered, double-buffered queue |
| `EventWriter<E>`       | `MessageWriter<M>`       | Write buffered messages         |
| `EventReader<E>`       | `MessageReader<M>`       | Read buffered messages          |
| `app.add_event::<E>()` | `app.add_message::<M>()` | Registration                    |

The old names still compile as **deprecated type aliases**.

`#[derive(Event)]` now exclusively means **observer/trigger-based** events.

### Messages (Buffered, Pull-Based)

Use for decoupled system-to-system communication. Messages persist for 2 frames (double-buffered).

```rust
#[derive(Message, Debug)]
struct DamageDealt {
    entity: Entity,
    amount: f32,
}

// Registration
app.add_message::<DamageDealt>();

// Sending
fn deal_damage(mut writer: MessageWriter<DamageDealt>) {
    writer.write(DamageDealt { entity, amount: 10.0 });
}

// Reading (multiple systems can read the same messages)
fn on_damage(mut reader: MessageReader<DamageDealt>) {
    for event in reader.read() {
        println!("Damage: {:?}", event);
    }
}
```

### Events (Observer/Trigger, Immediate)

Use for immediate callback-style responses. Can be global or entity-targeted.

```rust
#[derive(Event, Debug)]
pub struct HexSelectedEvent {
    pub position: HexPosition,
}

// Register observer
app.add_observer(|trigger: On<HexSelectedEvent>| {
    println!("Selected: {:?}", trigger.event().position);
});

// Trigger from a system
fn handle_click(mut commands: Commands, /* ... */) {
    commands.trigger(HexSelectedEvent { position: pos });
}
```

**Entity-scoped observers:**

```rust
commands.spawn((HexTile, HexPosition::new(0, 0)))
    .observe(|trigger: On<HexSelectedEvent>| {
        // Only fires for this specific entity
    });
```

### Which to Use?

| Use Case                                   | Pattern                                           |
| ------------------------------------------ | ------------------------------------------------- |
| System A tells System B something happened | `Message` + `MessageWriter`/`MessageReader`       |
| Immediate response to a discrete action    | `Event` + `commands.trigger()` + `add_observer()` |
| Entity-specific callback                   | `Event` + entity `.observe()`                     |
| Component lifecycle hooks                  | `On<Add, T>`, `On<Remove, T>` observers           |

### Current Hexorder Pattern

Hexorder currently uses the **observer pattern** (`Event` + `commands.trigger()`) for cross-feature
events like `HexSelectedEvent`. This is correct for immediate, callback-style responses. For future
features needing buffered multi-reader patterns, switch to `Message`.

---

## 5. Queries

### Basic Patterns

```rust
// Immutable iteration
for pos in &query { /* ... */ }

// Mutable iteration
for mut transform in &mut query { /* ... */ }

// Single entity (returns Result)
let Ok((mut transform, mut projection)) = query.single_mut() else {
    return;
};

// With Entity ID
for (entity, pos) in &query { /* ... */ }

// Collecting results
let results: Vec<_> = query.iter(app.world()).collect();
```

### Filters

```rust
// With/Without (include/exclude by component presence)
Query<&Health, With<Player>>
Query<&Health, Without<Enemy>>
Query<&Health, (With<Player>, Without<Enemy>)>

// Change detection
Query<&Health, Changed<Health>>    // mutated since last run
Query<(Entity, &Health), Added<Enemy>>  // newly added

// Or filter
Query<(&Health, &Armor), Or<(Changed<Health>, Changed<Armor>)>>
```

### Optional Components

```rust
fn show_info(query: Query<(&HexPosition, Option<&Terrain>)>) {
    for (pos, terrain) in &query {
        match terrain {
            Some(t) => println!("({},{}) = {:?}", pos.q, pos.r, t.terrain_type),
            None => println!("({},{}) = no terrain", pos.q, pos.r),
        }
    }
}
```

### Resource Change Detection

```rust
fn update_visuals(
    hovered: Res<HoveredHex>,
    selected: Res<SelectedHex>,
) {
    // Skip if nothing changed
    if !hovered.is_changed() && !selected.is_changed() {
        return;
    }
    // ...
}
```

### Test-Time Queries

```rust
// In tests, use world_mut() to create queries
let mut query = app.world_mut().query::<(&TopDownCamera, &Transform, &Projection)>();
let results: Vec<_> = query.iter(app.world()).collect();

// Filtered query in tests
let mut query = app.world_mut().query_filtered::<&Transform, With<HexTile>>();
let count = query.iter(app.world()).count();
```

---

## 6. Plugin Architecture

Every feature is a Bevy Plugin in its own module:

```rust
#[derive(Debug)]
pub struct MyFeaturePlugin;

impl Plugin for MyFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MyResource>()
            .add_systems(Startup, (setup_a, setup_b).chain())
            .add_systems(Update, (system_a, system_b));
    }
}
```

**Registration in main.rs:**

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "hexorder".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(hex_grid::HexGridPlugin)
        .add_plugins(camera::CameraPlugin)
        .run();
}
```

**Plugin Groups** (for related plugins):

```rust
pub struct MyPluginGroup;

impl PluginGroup for MyPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PluginA)
            .add(PluginB)
    }
}
```

---

## 7. State Management

```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
}

// Register
app.init_state::<GameState>();

// State-dependent systems
app.add_systems(Update, game_logic.run_if(in_state(GameState::Playing)));
app.add_systems(OnEnter(GameState::Playing), setup_game);
app.add_systems(OnExit(GameState::Playing), cleanup_game);

// Change state from a system
fn pause(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Paused);
}
```

**Transition order:** `OnExit(old)` -> `OnTransition { from, to }` -> `OnEnter(new)`

### SubStates

SubStates exist only when a parent state meets a condition:

```rust
#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(GameState = GameState::Playing)]
enum GamePhase {
    #[default]
    Setup,
    Combat,
    Resolution,
}

app.add_sub_state::<GamePhase>();
```

---

## 8. 3D Rendering

### Camera Setup

```rust
const CAMERA_Y: f32 = 100.0;

commands.spawn((
    TopDownCamera,                           // custom marker
    Camera3d::default(),                     // 3D camera
    Projection::Orthographic(OrthographicProjection {
        scale: 10.0,                         // zoom level
        near: 0.1,
        far: 1000.0,
        ..OrthographicProjection::default_3d()
    }),
    Transform::from_xyz(0.0, CAMERA_Y, 0.0)
        .looking_at(Vec3::ZERO, Vec3::Z),    // top-down, Z-up
));
```

**Projection is an enum, not a standalone component:**

```rust
// Updating projection at runtime
if let Projection::Orthographic(ref mut ortho) = *projection {
    ortho.scale = new_scale;
}
```

**Viewport to world ray (for mouse picking):**

```rust
let ray = camera.viewport_to_world(camera_transform, cursor_position).ok()?;
let distance = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
let world_pos = ray.get_point(distance);
```

### Built-in Camera Controllers (new in 0.18)

```rust
// Pan camera (2D-style WASD + scroll zoom)
app.add_plugins(PanCameraPlugin);
commands.spawn((Camera3d::default(), PanCamera::default()));

// Free camera (fly cam, useful for debugging)
app.add_plugins(FreeCameraPlugin);
commands.spawn((Camera3d::default(), FreeCamera::default()));
```

### Meshes

**Primitive shapes:**

| Shape       | Constructor                      |
| ----------- | -------------------------------- |
| `Cuboid`    | `Cuboid::new(w, h, d)`           |
| `Sphere`    | `Sphere::new(radius)`            |
| `Circle`    | `Circle::new(radius)`            |
| `Plane3d`   | `Plane3d::default()`             |
| `Cylinder`  | `Cylinder::new(radius, height)`  |
| `Capsule3d` | `Capsule3d::new(radius, height)` |
| `Torus`     | `Torus::new(inner, outer)`       |

```rust
commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
    Transform::from_xyz(0.0, 0.5, 0.0),
));
```

**Custom mesh (e.g., hexagons):**

```rust
use bevy::mesh::{Indices, PrimitiveTopology};

let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);  // Vec<[f32; 3]>
mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);       // Vec<[f32; 3]>
mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);             // Vec<[f32; 2]>
mesh.insert_indices(Indices::U32(indices));                    // Vec<u32>
```

Note: `Indices` and `PrimitiveTopology` are at `bevy::mesh::`, **not** `bevy::render::mesh::`.

### Materials (StandardMaterial / PBR)

```rust
materials.add(StandardMaterial {
    base_color: Color::srgb(0.8, 0.8, 0.8),
    metallic: 0.0,                    // 0 = dielectric, 1 = metal
    perceptual_roughness: 0.5,        // 0 = mirror, 1 = rough
    reflectance: 0.5,
    emissive: LinearRgba::BLACK,      // self-illumination
    unlit: false,                     // true = ignores lighting
    alpha_mode: AlphaMode::Opaque,
    ..default()
})
```

### Lighting

```rust
// Directional (sun)
commands.spawn((
    DirectionalLight {
        illuminance: light_consts::lux::OVERCAST_DAY,
        shadows_enabled: true,
        ..default()
    },
    Transform::default().looking_at(Vec3::new(-1.0, -1.0, -1.0), Vec3::Y),
));

// Point (omnidirectional)
commands.spawn((
    PointLight {
        intensity: 100_000.0,
        shadows_enabled: true,
        ..default()
    },
    Transform::from_xyz(4.0, 8.0, 4.0),
));

// Ambient (global fill)
commands.insert_resource(AmbientLight {
    color: Color::WHITE,
    brightness: 0.1,
});
```

### Gizmos (Debug Drawing)

```rust
fn debug_draw(mut gizmos: Gizmos) {
    gizmos.line(Vec3::ZERO, Vec3::new(1.0, 1.0, 0.0), GREEN);
    gizmos.circle(Isometry3d::IDENTITY, 1.0, GREEN);
    gizmos.sphere(Isometry3d::from_translation(Vec3::Y), 0.5, RED);
}
```

---

## 9. Input Handling

### Keyboard

```rust
fn keyboard_pan(keys: Res<ButtonInput<KeyCode>>, time: Res<Time>) {
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        // held down — continuous movement
    }
    if keys.just_pressed(KeyCode::Space) {
        // single press — discrete action
    }
    if keys.just_released(KeyCode::Escape) {
        // just released
    }
}
```

### Mouse Buttons

```rust
fn handle_click(mouse_buttons: Res<ButtonInput<MouseButton>>) {
    if mouse_buttons.just_pressed(MouseButton::Left) { /* click */ }
    if mouse_buttons.pressed(MouseButton::Middle) { /* drag */ }
}
```

### Mouse Motion and Scroll

Two approaches exist in Bevy 0.18:

**Accumulated resources** (simpler, used by Hexorder):

```rust
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};

fn mouse_pan(mouse_motion: Res<AccumulatedMouseMotion>) {
    let delta = mouse_motion.delta;  // Vec2
    if delta == Vec2::ZERO { return; }
    // ...
}

fn scroll_zoom(scroll: Res<AccumulatedMouseScroll>) {
    let scroll_amount = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y * 0.01,
    };
    // ...
}
```

**Message-based** (per-event granularity):

```rust
fn mouse_events(
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
) {
    for event in motion.read() {
        // event.delta: Vec2
    }
    for event in wheel.read() {
        // event.x, event.y, event.unit
    }
}
```

### Window Cursor Position

```rust
use bevy::window::PrimaryWindow;

fn get_cursor(windows: Query<&Window, With<PrimaryWindow>>) {
    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    // cursor_pos: Vec2 in window coordinates
}
```

---

## 10. Asset System

### Loading Assets

```rust
fn setup(asset_server: Res<AssetServer>) {
    let texture: Handle<Image> = asset_server.load("textures/ground.png");
    let scene: Handle<Scene> = asset_server.load("models/unit.glb#Scene0");
    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
}
```

### Creating Assets Programmatically

```rust
fn setup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh_handle: Handle<Mesh> = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mat_handle: Handle<StandardMaterial> = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        ..default()
    });
}
```

### Handle Types

- **Strong** handles (default): reference-counted, keep asset alive
- **Weak** handles: do not keep asset alive; asset may be unloaded
- When all strong handles are dropped, the asset is freed

### Storing Handles as Resources

```rust
#[derive(Resource, Debug)]
pub struct HexMaterials {
    pub default: Handle<StandardMaterial>,
    pub hovered: Handle<StandardMaterial>,
    pub selected: Handle<StandardMaterial>,
}
```

---

## 11. Transforms and Hierarchy

### Transform

```rust
// Constructors
Transform::from_xyz(1.0, 2.0, 3.0)
Transform::from_translation(Vec3::new(1.0, 2.0, 3.0))
Transform::from_rotation(Quat::from_rotation_y(FRAC_PI_4))
Transform::from_scale(Vec3::splat(2.0))

// Chained
Transform::from_xyz(0.0, 100.0, 0.0)
    .looking_at(Vec3::ZERO, Vec3::Z)
    .with_scale(Vec3::splat(0.5))

// Fields: translation (Vec3), rotation (Quat), scale (Vec3)
```

### GlobalTransform

Computed automatically by Bevy's transform propagation. Represents absolute world position. **Do not
mutate directly** — change `Transform` instead.

### Parent/Child Hierarchy

```rust
commands.spawn(Transform::default())
    .with_children(|parent| {
        parent.spawn((ChildComponent, Transform::from_xyz(1.0, 0.0, 0.0)));
        parent.spawn((ChildComponent, Transform::from_xyz(-1.0, 0.0, 0.0)));
    });
```

- Transform propagates from parent to children automatically
- Despawning a parent despawns all descendants

---

## 12. Picking

Bevy 0.18 has built-in mesh picking via `MeshPickingPlugin`:

```rust
use bevy::picking::mesh_picking::MeshPickingPlugin;

app.add_plugins((DefaultPlugins, MeshPickingPlugin));

commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    MeshMaterial3d(materials.add(Color::WHITE)),
))
.observe(|trigger: On<Pointer<Over>>| { /* hover enter */ })
.observe(|trigger: On<Pointer<Out>>| { /* hover exit */ })
.observe(|trigger: On<Pointer<Click>>| { /* clicked */ });
```

**Note:** Hexorder currently uses manual raycasting (`viewport_to_world` + plane intersection) for
hex picking. The built-in `MeshPickingPlugin` is available for simpler entity-level picking if
needed.

---

## 13. Testing Patterns

### Minimal Test App

```rust
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

// With asset support (needed for Mesh/Material handles)
fn test_app_with_assets() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app
}
```

### Testing a Startup System

```rust
#[test]
fn camera_spawns() {
    let mut app = test_app();
    app.init_resource::<CameraState>();
    app.add_systems(Startup, spawn_camera);
    app.update();

    let mut query = app.world_mut().query::<(&TopDownCamera, &Transform)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1);
}
```

### Testing Resource State

```rust
#[test]
fn grid_config_exists() {
    let mut app = test_app();
    app.add_systems(Startup, setup_grid_config);
    app.update();

    let config = app.world().get_resource::<HexGridConfig>();
    assert!(config.is_some());
    assert_eq!(config.unwrap().map_radius, 10);
}
```

### Testing Chained Systems

```rust
#[test]
fn bounds_adjust_with_config() {
    let mut app = test_app();
    app.init_resource::<CameraState>();
    app.insert_resource(HexGridConfig { /* ... */ });
    app.add_systems(Startup, (spawn_camera, configure_bounds).chain());
    app.update();

    let state = app.world().resource::<CameraState>();
    assert!(state.pan_bounds > 0.0);
}
```

### Testing Observer Events

Use `Arc<Mutex<Vec<T>>>` to capture triggered events:

```rust
use std::sync::{Arc, Mutex};

#[test]
fn click_fires_event() {
    let received = Arc::new(Mutex::new(Vec::<HexPosition>::new()));
    let received_clone = Arc::clone(&received);

    let mut app = test_app();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.insert_resource(SelectedHex::default());
    app.insert_resource(HoveredHex { position: Some(HexPosition::new(-1, 4)) });

    app.add_observer(move |trigger: On<HexSelectedEvent>| {
        received_clone.lock().unwrap().push(trigger.event().position);
    });

    app.add_systems(Update, handle_click);

    // Simulate input
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();

    let events = received.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], HexPosition::new(-1, 4));
}
```

### Simulating Input

```rust
// Mouse button
app.world_mut()
    .resource_mut::<ButtonInput<MouseButton>>()
    .press(MouseButton::Left);

// Keyboard
app.world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::KeyW);
```

### Counting Entities

```rust
let mut query = app.world_mut().query_filtered::<Entity, With<HexTile>>();
let count = query.iter(app.world()).count();
assert_eq!(count, expected);
```

---

## 14. Hexorder-Specific Conventions

### Project Architecture

```
src/
  main.rs              # App setup, plugin registration only
  contracts/           # Shared types (mirrors .specs/contracts/)
    mod.rs
    hex_grid.rs        # HexPosition, HexGridConfig, HexSelectedEvent, HexMoveEvent
    game_system.rs     # GameSystem, CellType, CellTypeRegistry, CellData, PropertyType, etc.
    editor_ui.rs       # EditorTool
  <feature>/
    mod.rs             # Plugin definition
    components.rs      # Feature-local components
    systems.rs         # Systems
    tests.rs           # Unit tests (#[cfg(test)])
```

### Contracts Protocol

Shared types between plugins live in `src/contracts/` and are specified in `.specs/contracts/`.

```rust
// src/contracts/mod.rs
#[allow(dead_code)]
pub mod hex_grid;
#[allow(dead_code)]
pub mod terrain;
```

`#[allow(dead_code)]` is used because not all contract types are consumed by all features at every
point in development.

**Before changing a contract:**

1. Propose in `.specs/coordination.md` under "Pending Contract Changes"
2. Update `.specs/contracts/<name>.md`
3. Implement in `src/contracts/<name>.rs`
4. Run `cargo build` to verify all consumers compile

### Cross-Feature Communication

Features communicate exclusively via Events and shared Resources from contracts. No direct imports
from other feature modules.

```rust
// Feature A fires:
commands.trigger(HexSelectedEvent { position: pos });

// Feature B observes:
app.add_observer(|trigger: On<HexSelectedEvent>| { /* ... */ });
```

### Coordinate System

- **Hex coordinates:** Axial (q, r) system via `HexPosition`
- **3D space:** Hex grid on XZ ground plane (Y = 0), camera looks down -Y axis
- **Hex math library:** `hexx` crate is canonical
- **Orientation:** Pointy-top hexagons

### Quality Gates

All code must pass before a feature is complete:

- `cargo test` — all tests pass
- `cargo clippy -- -D warnings` — zero warnings
- `cargo build` — compiles cleanly

### Code Rules (from constitution)

- No `unwrap()` in production code; use `?` or explicit error handling
- No `unsafe` without documented justification
- All public types derive `Debug` at minimum
- Components are data-only (no methods beyond trait impls)
- Logic lives in systems, not component methods
- Never use `World` directly in systems; use `Commands`, `Query`, `Res`

---

## 15. Bevy 0.18 Migration Notes

### From Pre-0.17 Code

| Old Pattern                             | New Pattern                                                   |
| --------------------------------------- | ------------------------------------------------------------- |
| `#[derive(Event)]` for buffered events  | `#[derive(Message)]`                                          |
| `EventWriter<E>`                        | `MessageWriter<M>`                                            |
| `EventReader<E>`                        | `MessageReader<M>`                                            |
| `app.add_event::<E>()`                  | `app.add_message::<M>()`                                      |
| `OrthographicProjection` as Component   | `Projection::Orthographic(OrthographicProjection { ... })`    |
| `EventReader<MouseWheel>`               | `Res<AccumulatedMouseScroll>` or `MessageReader<MouseWheel>`  |
| `EventReader<CursorMoved>`              | `Res<AccumulatedMouseMotion>` or `MessageReader<MouseMotion>` |
| `.after(system_fn)` in tuple            | `.chain()` or explicit `SystemSet`                            |
| `bevy::render::mesh::Indices`           | `bevy::mesh::Indices`                                         |
| `bevy::render::mesh::PrimitiveTopology` | `bevy::mesh::PrimitiveTopology`                               |

### Entity API Changes (0.17 → 0.18)

| Old                       | New                                             |
| ------------------------- | ----------------------------------------------- |
| `EntityRow`               | `EntityIndex`                                   |
| `Entity::row()`           | `Entity::index()`                               |
| `Entity::from_row()`      | `Entity::from_index()`                          |
| `Entities::flush()`       | Removed                                         |
| `Entities::alloc()`       | Removed                                         |
| `EntityDoesNotExistError` | `InvalidEntityError` / `EntityNotSpawnedError`  |
| `Commands::get_entity`    | Now includes non-spawned entities               |
| —                         | `Commands::get_spawned_entity` for spawned-only |

### EntityEvent Immutability (0.18)

`EntityEvent::from` and `EntityEvent::event_target_mut` moved to `SetEntityEventTarget` trait. All
`EntityEvent`s are now immutable by default.

### New in 0.18

- **Cargo feature collections:** `2d`, `3d`, `ui` for minimal builds
- **Built-in camera controllers:** `PanCameraPlugin`/`PanCamera`, `FreeCameraPlugin`/`FreeCamera`
- **Safe multi-component mutation:** `entity_mut.get_components_mut::<(&mut A, &mut B)>()`
- **Atmosphere occlusion:** Sunlight affected by procedural atmosphere
- **UI widgets:** `Popover`, `MenuPopup`, `ColorPlane`
- **Text styling:** `Strikethrough`, `Underline` components; `FontWeight` support
- **PBR fixes:** Corrected overly glossy/bright specular highlights

---

## 16. hexx 0.22 Reference

### HexLayout

```rust
let layout = hexx::HexLayout {
    orientation: hexx::HexOrientation::Pointy,
    ..hexx::HexLayout::default()
}
.with_hex_size(1.0);  // uniform size (sets scale to Vec2::splat(1.0))
```

**API changes from earlier hexx versions:**

- `HexLayout.hex_size` renamed to `HexLayout.scale` (now `Vec2` instead of `f32`)
- No `with_orientation()` method — set the `orientation` field directly
- Use `.with_hex_size(f32)` builder for uniform sizing

### HexPosition (Hexorder contract wrapper)

```rust
use crate::contracts::hex_grid::HexPosition;

let pos = HexPosition::new(3, -2);
let hex: hexx::Hex = pos.to_hex();
let back: HexPosition = HexPosition::from_hex(hex);
```

### Grid Generation

```rust
use hexx::Hex;

let hexes = Hex::ZERO.range(map_radius as u32);  // all hexes within radius
```

### World Position Conversion

```rust
let world_pos: Vec2 = layout.hex_to_world_pos(hex);  // returns Vec2 (x, y)
// For 3D: Transform::from_xyz(world_pos.x, 0.0, world_pos.y)

let hex: Hex = layout.world_pos_to_hex(Vec2::new(x, z));  // world to hex
```

### Hex Corners (for mesh generation)

```rust
let corners: [Vec2; 6] = layout.hex_corners(hex);
```

### Tile Count Formula

```rust
fn tile_count_for_radius(radius: u32) -> usize {
    if radius == 0 { return 1; }
    let r = radius as usize;
    1 + 6 * r * (r + 1) / 2
}
```

---

## 17. macOS Platform

### Graphics Backend

Bevy uses `wgpu` → **Metal** on macOS automatically. No Xcode project, Metal framework, or GPU
configuration needed. The rendering pipeline is GPU-accelerated out of the box.

### Input Quirks

**Scroll acceleration:** macOS applies OS-level scroll acceleration. Unlike other platforms that
send whole-number `Line` events (1.0 = one wheel step), macOS sends values from <0.1 (initial
movement) to >10.0 (fast flick). Hexorder handles both units:

```rust
let scroll_amount = match scroll.unit {
    MouseScrollUnit::Line => scroll.delta.y,       // desktop mouse
    MouseScrollUnit::Pixel => scroll.delta.y * 0.01, // trackpad / accelerated
};
```

**Touchpad gestures:** Bevy exposes macOS trackpad pinch-to-zoom and rotate gestures as events.
These could be wired to camera zoom/rotate in future milestones.

**Key mappings:** | macOS Key | Bevy KeyCode | |-----------|-------------| | Command (⌘) |
`SuperLeft` / `SuperRight` | | Option (⌥) | `AltLeft` / `AltRight` | | Control (⌃) | `ControlLeft` /
`ControlRight` |

### App Bundle

macOS apps are `.app` bundles — special directories that Finder displays as a single item.

```
Hexorder.app/
  Contents/
    Info.plist                  # Bundle metadata
    MacOS/
      hexorder                  # Executable
      assets/                   # Bevy assets (MUST be next to binary)
    Resources/
      hexorder.icns             # Dock/Finder icon
```

**Critical:** Bevy expects the `assets/` folder in `Contents/MacOS/` alongside the binary — **not**
in `Contents/Resources/`. Apple doesn't enforce the convention, so this works.

**Build:** `./build/macos/bundle.sh [--release] [--universal]`

### Universal Binaries (Intel + Apple Silicon)

```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
./build/macos/bundle.sh --release --universal
```

This compiles for both architectures and combines them with `lipo`.

### Important: No dynamic_linking

The Bevy `dynamic_linking` cargo feature must **not** be enabled when building app bundles. It's
designed for faster dev iteration and will not work in a standalone `.app`.

### DMG Creation (for distribution)

**Using `create-dmg`** (install: `brew install create-dmg`):

```bash
create-dmg \
  --volname "Hexorder" \
  --volicon "assets/icon/hexorder.icns" \
  --window-size 600 400 \
  --icon "Hexorder.app" 150 200 \
  --app-drop-link 450 200 \
  "Hexorder.dmg" \
  "target/release/Hexorder.app"
```

**Using native `hdiutil`:**

```bash
hdiutil create -fs HFS+ \
  -volname "Hexorder" \
  -srcfolder "target/release/Hexorder.app" \
  "Hexorder.dmg"
```

### Code Signing and Notarization

Required for distribution outside the Mac App Store. Not needed during development.

```bash
# Sign the bundle
codesign --force --deep --sign "Developer ID Application: Your Name (TEAM_ID)" \
  target/release/Hexorder.app

# Submit for notarization
xcrun notarytool submit Hexorder.dmg \
  --apple-id "you@example.com" \
  --team-id "TEAM_ID" \
  --password "@keychain:AC_PASSWORD" \
  --wait

# Staple the ticket
xcrun stapler staple Hexorder.dmg
```

### Known Issues

**Window management apps:** Apps like Magnet, Rectangle, or Amethyst can cause window dragging lag
with Bevy. This is a [winit bug](https://github.com/rust-windowing/winit/issues/1737). Workaround:
close the window manager if lag occurs.

---

## 18. Common Pitfalls

### 1. OrthographicProjection Is Not a Component

```rust
// WRONG: Query<&OrthographicProjection>
// RIGHT:
Query<&Projection>

// Then match:
if let Projection::Orthographic(ref mut ortho) = *projection {
    ortho.scale = new_value;
}
```

### 2. System Ordering on Bare Functions

```rust
// WRONG: .after(my_system_fn) on bare function pointers
// RIGHT: use .chain() or SystemSet
app.add_systems(Update, (system_a, system_b).chain());
```

### 3. Event vs Message Confusion

```rust
// For observer/trigger (immediate): use Event
#[derive(Event, Debug)]
struct MyEvent;

// For buffered system-to-system (pull-based): use Message
#[derive(Message, Debug)]
struct MyMessage;
```

### 4. Mesh Module Path

```rust
// WRONG: use bevy::render::mesh::{Indices, PrimitiveTopology};
// RIGHT:
use bevy::mesh::{Indices, PrimitiveTopology};
```

### 5. Query::single() Returns Result

```rust
// Always handle the error case
let Ok(result) = query.single() else { return; };
let Ok((mut transform, mut proj)) = query.single_mut() else { return; };
```

### 6. Commands Are Deferred

Spawning/despawning/inserting via `Commands` doesn't take effect immediately. If you need the entity
to exist in the same system, use `World` directly (exclusive system) or structure ordering so the
consuming system runs after a sync point.

### 7. hexx Scale Is Vec2

```rust
// WRONG: layout.hex_size = 1.0;
// RIGHT:
let layout = HexLayout::default().with_hex_size(1.0);
// Or: layout.scale = Vec2::splat(1.0);
```

### 8. Resource Must Exist Before Access

`Res<T>` will panic if the resource doesn't exist. Use `Option<Res<T>>` when a resource may not be
inserted yet:

```rust
fn safe_system(config: Option<Res<HexGridConfig>>) {
    if let Some(config) = config {
        // use config
    }
}
```

### 9. Change Detection Ticks on First Run

`Res::is_changed()` returns `true` on the first system run after the resource is inserted. Account
for this in systems that should only react to actual mutations.

### 10. Handle Cloning

`Handle<T>` is reference-counted. `.clone()` is cheap (increments refcount) and is the correct way
to share handles across entities:

```rust
for hex in hexes {
    commands.spawn((
        Mesh3d(mesh_handle.clone()),      // same mesh, shared
        MeshMaterial3d(mat_handle.clone()), // same material, shared
    ));
}
```

---

## 19. GPU Pipeline Startup and Window Flash Prevention

### The Problem

When a Bevy app launches, the OS creates and displays a window before the GPU has rendered its first
frame. On macOS this produces a brief white (or light gray) flash from the default NSWindow
background color. Additionally, wgpu compiles shaders (WGSL → MSL → GPU machine code) at runtime
during the first few frames, which can cause visible stutter.

**Root cause chain:**

1. OS creates window → paints default background (white)
2. wgpu initializes Metal device
3. Bevy queues render pipelines → wgpu compiles shaders asynchronously
4. First rendered frame appears (ClearColor or scene content)

The gap between steps 1 and 4 is the flash.

### Solution: Hidden Window + Delayed Reveal (Hexorder's Approach)

Start the window hidden, let the GPU render a few dark frames, then reveal:

```rust
// In WindowPlugin configuration:
Window {
    visible: false,
    window_theme: Some(WindowTheme::Dark),
    ..default()
}

// Dark clear color so first rendered frame matches theme
app.insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.04)));

// Reveal after 3 frames
fn reveal_window(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut frames: Local<u32>,
    mut done: Local<bool>,
) {
    if *done { return; }
    *frames += 1;
    if *frames >= 3 {
        if let Ok(mut window) = windows.single_mut() {
            window.visible = true;
        }
        *done = true;
    }
}
```

This is Bevy's officially recommended pattern (see `examples/window/window_settings.rs`).

### Why 3 Frames?

Frame 0: Bevy initializes render device, creates GPU surface. Frame 1: First render pass executes
with ClearColor. Frame 2: Render pipeline stabilizes. Frame 3: Safe to show — the dark ClearColor
has been applied to the window surface.

The count is a practical heuristic, not an exact science. 3 frames works reliably across macOS
hardware.

### Alternative Approaches We Evaluated

#### Splash Screen Overlay (UI-based)

Spawn a full-screen `Node` with `GlobalZIndex(999)` and `BackgroundColor` to cover the scene while
pipelines compile. Dismiss after readiness check or timer.

**Verdict:** Adds complexity without benefit if the hidden-window approach already eliminates the
flash. Only useful if you need to display branded loading content (logo, progress bar) to the user.
Avoid using a fixed timer — use pipeline readiness instead (see below).

#### Two-Window Splash (Separate Native-Feel Splash)

Spawn a second borderless `Window` entity with a `Camera2d` and
`RenderTarget::Window(WindowRef::Entity(...))` for the splash, keep the primary window hidden.

**Verdict:** Does not work well with bevy_egui. When the primary window is hidden,
`EguiContexts::ctx_mut()` either returns `Err` or routes to the splash window. The `in_state()` run
condition also does not work reliably in the `EguiPrimaryContextPass` schedule. Despawning the
splash window produces `bevy_winit` warnings for stale window events. Avoid this approach.

#### Native NSWindow (Pre-Engine Splash)

Create an Objective-C NSWindow via `objc2-app-kit` FFI before Bevy's event loop starts. This is how
Unreal Engine's platform splash works (Phase 1: native OS window, Phase 2: engine loading screen).

**Verdict:** Eliminates the flash completely but requires unsafe FFI, platform-specific code, and
careful coordination with winit's event loop ownership. Overkill for a design tool. Reserve for
consumer game distribution if needed.

### Pipeline Readiness Checking

The `bevy_pipelines_ready` crate (v0.8 for Bevy 0.18) exposes a `PipelinesReady` resource that
tracks how many render pipelines have finished compiling:

```rust
use bevy_pipelines_ready::PipelinesReady;

fn check_ready(pipelines: Res<PipelinesReady>) {
    let count = pipelines.get(); // number of ready pipelines
}
```

**Stability pattern:** Don't transition on the first frame where count > 0. Wait for the count to
stop changing for several consecutive frames (new entities can trigger new pipeline creation):

```rust
fn dismiss_when_ready(
    pipelines: Res<PipelinesReady>,
    mut last_count: Local<usize>,
    mut stable_frames: Local<u32>,
) {
    let current = pipelines.get();
    if current > 0 && current == *last_count {
        *stable_frames += 1;
    } else {
        *stable_frames = 0;
        *last_count = current;
    }
    if *stable_frames >= 5 {
        // pipelines are stable — safe to show content
    }
}
```

This replaces arbitrary fixed timers (e.g., "wait 5 seconds") with actual GPU readiness. Useful if
you add a splash or loading screen in the future.

### Shader Compilation on macOS

**Key facts:**

- wgpu translates WGSL → MSL (Metal Shading Language) at runtime. This cannot be pre-compiled at
  build time.
- Metal's driver (`MTLCompilerService`) compiles MSL → GPU machine code. This is cached system-wide
  by macOS after first run.
- `synchronous_pipeline_compilation` in Bevy has no effect on macOS (Metal always compiles
  asynchronously via its driver).
- Second and subsequent launches are significantly faster because Metal reuses its shader cache.
- `MTLBinaryArchive` (Apple API) allows shipping pre-compiled GPU binaries, reducing first-launch
  compilation from ~86s to ~3s in extreme cases. Not exposed through wgpu.

**Practical implication for Hexorder:** First launch on a new machine is the slowest. Subsequent
launches benefit from Metal's automatic cache. No action needed — the hidden-window reveal covers
the first-launch delay.

### bevy_egui Interactions

**`EguiPrimaryContextPass` and hidden windows:** When the primary window starts hidden, bevy_egui
may not fully initialize its context. The egui rendering systems should run unconditionally (no
`in_state()` gating in `EguiPrimaryContextPass`) and use early-return on `ctx_mut()` failure:

```rust
pub fn my_egui_system(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    // ... render UI
}
```

**`in_state()` in `EguiPrimaryContextPass`:** State-based run conditions do not work reliably in
this schedule. If you need to gate egui systems, check state inside the system body or use a
resource flag instead.

**`configure_theme` should run every frame:** Don't use a `Local<bool>` one-shot guard for theme
configuration. If the egui context resets (e.g., after window visibility changes), the theme must be
re-applied. The cost is negligible (a few struct assignments per frame).

### Summary: What Works

| Technique                             | Solves White Flash     | Solves Shader Stutter | Complexity                   |
| ------------------------------------- | ---------------------- | --------------------- | ---------------------------- |
| Hidden window + reveal after 3 frames | Yes                    | No                    | Low                          |
| ClearColor + WindowTheme::Dark        | Reduces flash severity | No                    | Trivial                      |
| UI overlay splash                     | No (covers it)         | No (covers it)        | Medium                       |
| Pipeline readiness check              | No                     | Yes (smart dismissal) | Low-Medium                   |
| Two-window splash                     | Partially              | No                    | High (broken with bevy_egui) |
| Native NSWindow pre-engine            | Yes (completely)       | No                    | Very High                    |
| Metal shader cache                    | N/A                    | Yes (on re-launch)    | Zero (automatic)             |

**Hexorder's current approach:** Hidden window + dark ClearColor + reveal after 3 frames. Simplest
approach that fully solves the white flash. All systems run from frame 1 with no gating.

### References

- [Bevy window_settings example](https://github.com/bevyengine/bevy/blob/main/examples/window/window_settings.rs)
  — official hidden-window pattern
- [Bevy Issue #9771](https://github.com/bevyengine/bevy/issues/9771) — white frame on startup
- [bevy_pipelines_ready](https://github.com/rparrett/bevy_pipelines_ready) — pipeline readiness
  tracking (v0.8 = Bevy 0.18)
- [Bevy loading_screen example](https://bevy.org/examples/games/loading-screen/) — pipeline-ready
  loading screen
- [Bevy Issue #13354](https://github.com/bevyengine/bevy/issues/13354) — API for precompiling
  shaders (open)
- [WWDC20 — Build GPU binaries with Metal](https://developer.apple.com/videos/play/wwdc2020/10615/)
  — MTLBinaryArchive
- [Unreal Engine Preload Screens](https://unrealist.org/engine-startup-preload-screens/) — two-phase
  native + engine splash
- [Godot pipeline compilations](https://docs.godotengine.org/en/stable/tutorials/performance/pipeline_compilations.html)
  — ubershader approach
