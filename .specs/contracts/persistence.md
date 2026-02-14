# Contract: Persistence

## Owner

`persistence` feature plugin (M5)

## Purpose

Types for saving and loading game system definitions and board state to `.hexorder` (RON) files.

## Types

### `GameSystemFile`

Top-level container for a saved game system + board state.

| Field            | Type                 | Description                     |
| ---------------- | -------------------- | ------------------------------- |
| `format_version` | `u32`                | File format version (migration) |
| `game_system`    | `GameSystem`         | Game system metadata            |
| `entity_types`   | `EntityTypeRegistry` | All entity types + enum defs    |
| `concepts`       | `ConceptRegistry`    | Concepts + bindings             |
| `relations`      | `RelationRegistry`   | Relations                       |
| `constraints`    | `ConstraintRegistry` | Constraints                     |
| `map_radius`     | `u32`                | Hex grid radius                 |
| `tiles`          | `Vec<TileSaveData>`  | Per-tile cell data              |
| `units`          | `Vec<UnitSaveData>`  | Placed unit data                |

### `TileSaveData`

Serialized form of a hex tile's cell data.

| Field            | Type                             | Description             |
| ---------------- | -------------------------------- | ----------------------- |
| `position`       | `HexPosition`                    | Hex coordinates         |
| `entity_type_id` | `TypeId`                         | Cell type               |
| `properties`     | `HashMap<TypeId, PropertyValue>` | Per-instance properties |

### `UnitSaveData`

Serialized form of a placed unit.

| Field            | Type                             | Description             |
| ---------------- | -------------------------------- | ----------------------- |
| `position`       | `HexPosition`                    | Hex coordinates         |
| `entity_type_id` | `TypeId`                         | Unit type               |
| `properties`     | `HashMap<TypeId, PropertyValue>` | Per-instance properties |

### `PersistenceError`

Error type for save/load operations.

| Variant              | Fields                     | Description                 |
| -------------------- | -------------------------- | --------------------------- |
| `Io`                 | `std::io::Error`           | File system error           |
| `Serialize`          | `ron::Error`               | RON serialization failure   |
| `Deserialize`        | `ron::error::SpannedError` | RON deserialization failure |
| `UnsupportedVersion` | `found: u32, max: u32`     | Unknown format version      |

### `AppScreen`

Application screen state.

| Variant    | Description                            |
| ---------- | -------------------------------------- |
| `Launcher` | Startup screen — new/open project      |
| `Editor`   | Main editor — all editing tools active |

### `CurrentFilePath`

Tracks the path to the currently open file.

| Field  | Type              | Description         |
| ------ | ----------------- | ------------------- |
| `path` | `Option<PathBuf>` | None if never saved |

### `PendingBoardLoad`

Temporary resource for deferred board state application after load.

| Field   | Type                | Description        |
| ------- | ------------------- | ------------------ |
| `tiles` | `Vec<TileSaveData>` | Tile data to apply |
| `units` | `Vec<UnitSaveData>` | Units to spawn     |

### `SaveRequestEvent`

Triggers a save operation.

| Field     | Type   | Description                          |
| --------- | ------ | ------------------------------------ |
| `save_as` | `bool` | If true, always show the file dialog |

### `LoadRequestEvent`

Triggers a load operation. No fields.

### `NewProjectEvent`

Triggers creation of a new empty project. No fields.

## Functions

### `save_to_file(path: &Path, data: &GameSystemFile) -> Result<(), PersistenceError>`

Serialize a `GameSystemFile` to RON and write to disk.

### `load_from_file(path: &Path) -> Result<GameSystemFile, PersistenceError>`

Read a RON file from disk and deserialize to `GameSystemFile`.

## Consumed By

- `persistence` plugin — save/load systems
- `editor_ui` plugin — launcher screen, file menu

## Dependencies

- `game_system` contract — `GameSystem`, `EntityTypeRegistry`, `TypeId`, `PropertyValue`
- `ontology` contract — `ConceptRegistry`, `RelationRegistry`, `ConstraintRegistry`
- `hex_grid` contract — `HexPosition`
