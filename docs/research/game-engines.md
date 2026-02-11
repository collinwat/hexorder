# Game Engine Property Type Research

## Research Prompt

> Research what property/attribute data types are supported by major game engines and game system
> design tools for entity properties. The goal is to understand what types are commonly available
> when defining custom properties on game entities (like terrain types, unit types, etc.) in the
> context of building Hexorder — a game system design tool for tabletop war board games.

## Date

2026-02-08

## Context

Hexorder M2 introduces a user-defined property system. Properties are entity-agnostic — they attach
to any entity (vertices, units, etc.) as defined by the Game System. This research informs which
primitive data types the property system should support.

---

## Findings by Engine

### 1. Unity — Serializable Field Types (ScriptableObjects)

**Primitives:**

- `int`, `float`, `double`, `bool`, `string`, `char`

**Math/Spatial:**

- `Vector2`, `Vector3`, `Vector4` (and integer variants `Vector2Int`, `Vector3Int`)
- `Quaternion`, `Matrix4x4`
- `Rect`, `RectInt`, `Bounds`, `BoundsInt`

**Visual/Design:**

- `Color`, `Color32`
- `AnimationCurve`, `Gradient`
- `LayerMask`

**Enums:**

- Any C# `enum` type

**Collections:**

- `T[]` (arrays of any serializable type)
- `List<T>`

**References:**

- References to any `UnityEngine.Object` subclass (GameObjects, ScriptableObjects, Textures,
  Materials, AudioClips, etc.)

**Custom Structs/Classes:**

- Any class or struct marked `[System.Serializable]` (serialized inline, not by reference; no
  polymorphism; max 7 levels deep)

**References:**

- [Unity Script Serialization Manual](https://docs.unity3d.com/550/Documentation/Manual/script-Serialization.html)
- [ScriptableObject Manual](https://docs.unity3d.com/Manual/class-ScriptableObject.html)

---

### 2. Unreal Engine — UPROPERTY System

**Primitives:**

- `bool`, `uint8`/`int8`, `int32`/`uint32`, `int64`/`uint64`, `float`, `double`

**String Types:**

- `FString` (dynamic character array)
- `FName` (immutable, case-insensitive identifier — fast comparison)
- `FText` (localization-aware display text)

**Math/Spatial:**

- `FVector` (3D), `FVector2D` (2D), `FRotator`, `FQuat`, `FTransform`

**Visual:**

- `FColor` (8-bit RGBA), `FLinearColor` (floating-point RGBA)

**Containers:**

- `TArray<T>` (dynamic array)
- `TMap<K,V>` (key-value dictionary)
- `TSet<T>` (unique value set)

**References:**

- `UObject*` pointers (hard references)
- `TSubclassOf<T>` (class picker, restricted to subclasses)
- `TSoftObjectPtr<T>` (soft/lazy reference — path-based, loaded on demand)
- `TWeakObjectPtr<T>` (weak reference)

**Enums:**

- Any `UENUM()` declared enum (including Blueprint-exposed)

**Structs:**

- Any `USTRUCT()` declared struct (fully nestable, editable in details panel)

**Metadata/Validation:**

- `EditAnywhere`, `BlueprintReadWrite`, `Category`, `ClampMin`/`ClampMax`, `DisplayName`, `SaveGame`
  — control editor behavior and validation

**References:**

- [Unreal Engine UProperties Documentation](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-engine-uproperties)
- [Blueprint Variables](https://dev.epicgames.com/documentation/en-us/unreal-engine/blueprint-variables-in-unreal-engine)
- [Unreal Garden UPROPERTY Specifiers](https://unreal-garden.com/docs/uproperty/)

---

### 3. Godot — @export Property Types

**Primitives:**

- `int`, `float`, `bool`, `String`

**Spatial:**

- `Vector2`, `Vector2i`, `Vector3`, `Vector3i`, `Vector4`

**Visual:**

- `Color`, `@export_color_no_alpha` (RGB only)

**Numeric Annotations:**

- `@export_range(min, max, step)` with hints: `"or_less"`, `"or_greater"`, `"exp"`, `"hide_slider"`,
  `"suffix"`, `"radians_as_degrees"`
- `@export_exp_easing` (visual easing curve)

**Selection/Choice:**

- `@export_enum(...)` (dropdown choice from named options)
- `@export_flags(...)` (bitfield multi-select)
- `@export_flags_2d_physics`, `@export_flags_3d_physics` (layer masks)

**Text:**

- `@export_multiline` (multi-line string)
- `@export_file` / `@export_dir` (file/directory path picker)

**Collections:**

- `Array[T]` (typed arrays)
- `PackedStringArray`, `PackedVector3Array`, `PackedColorArray`, etc.
- Dictionaries (via typed exports)

**Organization:**

- `@export_group`, `@export_subgroup`, `@export_category` (inspector grouping)

**References:**

- [GDScript Exported Properties](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/gdscript_exports.html)
- [GitHub godot-docs exports.rst](https://github.com/godotengine/godot-docs/blob/master/tutorials/scripting/gdscript/gdscript_exports.rst)

---

### 4. Tabletop Simulator — Object Properties

TTS uses Lua scripting with a relatively simple property model:

**Built-in Object Properties:**

- `string`: name, guid, memo, type, description
- `bool`: locked, interactable, use_gravity, resting, hide_when_face_down, ignore_fog_of_war
- `float/numeric`: mass, drag, angular_drag, bounciness, dynamic_friction, static_friction, value
- `Vector` (table with x,y,z): position, rotation, scale, velocity
- `Color` (table with r,g,b): player color, object tint

**Custom Data Mechanisms:**

- `memo` field: a single persistent string for user data
- `script_state`: JSON blob saved/loaded via `onSave()`/`onLoad()` callbacks (arbitrarily complex)
- `Tags`: string-based classification system (array of strings)
- Lua variables: any Lua type in attached scripts

**Object-Type-Specific:**

- Integer enums for object type (Generic=0, Figurine=1, Dice=2, etc.)
- Integer enums for material (Plastic=0, Wood=1, Metal=2, Cardboard=3)

TTS has no formal "property definition schema" — it is freeform Lua scripting with JSON persistence.

**References:**

- [TTS Object API](https://api.tabletopsimulator.com/object/)

---

### 5. Foundry VTT — DataModel Field Types

Foundry provides the richest structured schema system for tabletop game system definitions, with
**34 field types**:

**Primitives:**

- `BooleanField`, `NumberField`, `StringField`, `IntegerSortField`

**Specialized Numeric:**

- `AngleField` (0-360), `AlphaField` (0.0-1.0), `HueField`

**Visual:**

- `ColorField`

**Text/Rich Content:**

- `HTMLField`, `JavaScriptField`, `JSONField`

**File/Path:**

- `FilePathField` (media file paths with category restrictions)

**Identity/References:**

- `DocumentIdField`, `DocumentUUIDField`, `ForeignDocumentField`
- `DocumentAuthorField`, `DocumentTypeField`, `DocumentOwnershipField`
- `DocumentFlagsField`, `DocumentStatsField`

**Composite/Nested:**

- `SchemaField` (structured nested data with its own schema)
- `ObjectField` (freeform object data)
- `TypeDataField`, `TypedObjectField`, `TypedSchemaField`
- `EmbeddedDataField`, `EmbeddedDocumentField`
- `EmbeddedCollectionField`, `EmbeddedCollectionDeltaField`

**Collections:**

- `ArrayField`, `SetField`

**Other:**

- `AnyField` (accepts any type), `DataField` (base), `ShaderField`

**References:**

- [Foundry VTT System Data Models](https://foundryvtt.com/article/system-data-models/)
- [Foundry VTT API - fields module](https://foundryvtt.com/api/modules/foundry.data.fields.html)
- [Foundry VTT Community Wiki - DataModel](https://foundryvtt.wiki/en/development/api/DataModel)

---

### 6. VASSAL — Game Piece Property System

VASSAL is purpose-built for wargames and has the most domain-specific property system:

**Property Trait Types:**

- **Marker** — named property with a fixed, immutable value (set at piece creation)
- **Dynamic Property** — named property that can change during gameplay (via commands, expressions,
  key commands)
- **Calculated Property** — value derived from a formula/expression referencing other properties
- **Property Sheet** — a set of independent values managed together on a single piece
- **Spreadsheet** — tabular data on a piece
- **Global Property** — module/map/zone-wide shared variable
- **Set Global Property** — trait that writes to a global property
- **Set Piece Property** — trait that writes to other pieces' dynamic properties

**Underlying Data Types:** VASSAL properties are fundamentally **string-typed** with implicit
coercion:

- Text strings (the default)
- Numeric values (strings that contain numbers; compared with `<`, `<=`, `>`, `>=`)
- Boolean values (strings "true" or "false")
- Empty string "" (default when property does not exist)

**Property Sheet Field Formats (8 types):**

1. Text (single-line)
2. Multi-line text
3. Label Only (read-only display)
4. Tick Marks (checkboxes for tracking values)
5. Tick Marks with Max Field
6. Tick Marks with Value Field
7. Tick Marks with Value and Max
8. Spinner (numeric with increment/decrement buttons)

**Expression System:**

- Operators: `==`, `!=`, `=~` (regex match), `<`, `<=`, `>`, `>=`
- Logical: `&&`, `||`
- Properties can reference other properties by name in expressions
- BeanShell expressions for complex calculations

**Prototype System:**

- Prototypes serve as templates (like class inheritance for pieces)
- A piece can inherit traits from a Prototype Definition

**References:**

- [VASSAL Properties Reference](https://vassalengine.org/doc/latest/ReferenceManual/Properties.html)
- [VASSAL Game Piece Reference](https://vassalengine.org/doc/latest/ReferenceManual/GamePiece.html)
- [VASSAL Property Sheet Reference](https://vassalengine.org/doc/latest/ReferenceManual/PropertySheet.html)
- [VASSAL Prototypes Reference](https://vassalengine.org/doc/latest/ReferenceManual/Prototypes.html)

---

## Synthesis: Common Set Across Engines

| Type Category     | Specific Type                               | Present In                                    |
| ----------------- | ------------------------------------------- | --------------------------------------------- |
| **Boolean**       | true/false                                  | All 6                                         |
| **Integer**       | Whole numbers                               | All 6                                         |
| **Float/Number**  | Decimal numbers                             | All 6                                         |
| **String/Text**   | Free-form text                              | All 6                                         |
| **Enum/Choice**   | Selection from a fixed set of named options | Unity, Unreal, Godot, Foundry, VASSAL         |
| **Color**         | RGB or RGBA color value                     | Unity, Unreal, Godot, TTS, Foundry            |
| **Array/List**    | Ordered collection of values                | Unity, Unreal, Godot, Foundry, TTS (via JSON) |
| **Reference**     | Pointer/link to another entity/object       | Unity, Unreal, Godot, Foundry                 |
| **Struct/Schema** | Nested composite of typed fields            | Unity, Unreal, Godot, Foundry                 |

### Universal Core (all or nearly all):

1. **Boolean** — yes/no flags (e.g., `is_impassable`, `can_fly`)
2. **Integer** — whole numbers (e.g., `movement_cost`, `attack_strength`, `range`)
3. **Float** — decimal numbers (e.g., `defense_modifier`, `probability`)
4. **String** — text (e.g., `unit_name`, `description`, `flavor_text`)
5. **Enum/Choice** — selection from options (e.g., `terrain_class`, `unit_type`, `era`)

### Strong Consensus (4-5 of 6):

6. **Color** — for visual representation
7. **Array/List** — ordered collection (e.g., list of abilities)
8. **Reference to Another Entity** — linking entities (e.g., unit -> weapon_type)
9. **Nested Struct/Schema** — composite sub-objects (e.g., "combat stats" block)

### Notable Game Engine Types Not in Tabletop Tools:

10. **Vector2/Vector3** — spatial coordinates (Unity, Unreal, Godot)
11. **Map/Dictionary** — key-value pairs (Unreal `TMap`, Foundry `ObjectField`)

---

## Hexorder-Specific Recommendations

### High Priority (directly needed for tabletop wargame design):

| Type             | Purpose                                 | Wargame Example                               |
| ---------------- | --------------------------------------- | --------------------------------------------- |
| `Int`            | Whole number values                     | `movement_cost`, `attack_strength`, `range`   |
| `Float`          | Decimal values                          | `defense_modifier`, `probability`             |
| `Bool`           | Binary flags                            | `is_impassable`, `blocks_los`                 |
| `String`         | Free text                               | `name`, `description`                         |
| `Enum`           | Choice from defined set                 | `terrain_class`, `unit_type`, `movement_mode` |
| `Color`          | Visual representation                   | `map_color`, `faction_color`                  |
| `IntRange`       | Bounded integer (min/max)               | `morale` (1-10), `strength` (0-20)            |
| `FloatRange`     | Bounded float (min/max)                 | `modifier` (-1.0 to 1.0)                      |
| `EntityRef`      | Reference to another defined type       | unit -> weapon_type, hex -> terrain_type      |
| `List<T>`        | Ordered collection                      | `special_abilities`, `allowed_terrain`        |
| `Map<Enum, Int>` | Enum-keyed lookup table                 | movement_cost per movement_type, CRT column   |
| `Flags`          | Bitflag set from enum                   | unit capabilities, terrain features           |
| `Struct`         | Named group of sub-properties           | `CombatProfile { attack, defense, range }`    |
| `Formula`        | Expression referencing other properties | `effective_strength = base * morale_mod`      |
| `AssetPath`      | Path to image/model asset               | counter art, terrain texture                  |

### Key Insight: `Map<Enum, Int/Float>`

The `Map<Enum, Int>` (or `Map<Enum, Float>`) type stands out as **critical for wargames**. The
classic Combat Results Table (CRT), terrain effects charts, and movement cost tables are all
fundamentally enum-keyed lookup tables. Most general-purpose engines support this at some level
(Unreal: `TMap`, Foundry: `ObjectField`), but tabletop-specific tools like VASSAL handle it through
their expression system instead of a first-class type.

### Key Insight: Calculated/Derived Properties

VASSAL's Calculated Property pattern is directly relevant to Hexorder. Wargame designers need
derived values like `effective_strength = base_strength * morale_modifier * terrain_modifier`. This
should be considered for M2 or a near-follow milestone even if the full rules engine is M4.

### Medium Priority (useful for richer system design):

| Type                      | Purpose                     | Notes                                                               |
| ------------------------- | --------------------------- | ------------------------------------------------------------------- |
| `Nested Struct/Schema`    | Grouping sub-properties     | Avoids flat lists; e.g., `CombatProfile { attack, defense, range }` |
| `Bitflags / Flag Set`     | Multi-boolean capabilities  | Godot `@export_flags`, Unreal bitmask                               |
| `Calculated Property`     | Derived from formula        | VASSAL pattern; `strength = base * modifier`                        |
| `Expression / Formula`    | Complex calculations        | VASSAL BeanShell; CRT column = attacker/defender                    |
| `Localized Text`          | Display vs internal strings | Unreal `FText` vs `FName`; useful for export                        |
| `File/Image Path`         | Asset association           | Counter art, terrain textures                                       |
| `Angle`                   | Facing rules                | Hex-side facing in wargames                                         |
| `Set (unique collection)` | Unique values               | "set of terrain types this unit can enter"                          |
| `Tick Marks / Tracker`    | Resource tracking UI        | VASSAL-specific; ammo, fuel, morale tracking                        |

---

## M2 Scope Recommendation

For M2 ("The World Has Properties"), implement the **universal core** plus the types most critical
for the first use case (vertex/terrain properties):

**M2 types**: `Bool`, `Int`, `Float`, `String`, `Color`, `Enum`

**M3+ types** (when entities and relationships are needed): `EntityRef`, `List<T>`, `Map<K,V>`,
`Struct`, `IntRange`, `FloatRange`, `Flags`, `Formula`, `AssetPath`

This gives enough expressiveness for defining vertex types with meaningful properties while keeping
the type system implementation manageable. The architecture should be extensible so new types can be
added without restructuring.
