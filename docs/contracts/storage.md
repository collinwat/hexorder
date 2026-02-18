# Contract: Storage

## Owner

`persistence` plugin

## Purpose

Storage abstraction layer. Defines a trait for I/O backends, configuration resolved from build
target, and a Bevy resource wrapper. Systems use the `StorageProvider` trait through the `Storage`
resource instead of calling file I/O helpers directly.

## Types

### `StorageSource`

How the base directory was determined.

| Variant        | Description                                             |
| -------------- | ------------------------------------------------------- |
| `MacOs`        | `macos` feature flag — `~/Library/Application Support/` |
| `Xdg`          | `xdg` feature flag — `$XDG_DATA_HOME/hexorder/`         |
| `ProjectLocal` | Default dev mode — `.state/{version}/` in project root  |

### `StorageConfig`

Resolved storage configuration.

| Field      | Type            | Description                         |
| ---------- | --------------- | ----------------------------------- |
| `base_dir` | `PathBuf`       | Base directory for saved projects   |
| `source`   | `StorageSource` | How the base directory was resolved |

### `ProjectEntry`

Metadata about a saved project on disk.

| Field  | Type      | Description                          |
| ------ | --------- | ------------------------------------ |
| `name` | `String`  | Human-readable name (from file stem) |
| `path` | `PathBuf` | Full path to the `.hexorder` file    |

### `StorageProvider` (trait)

Object-safe trait for storage backends. `Send + Sync`.

| Method     | Signature                                                       | Description                           |
| ---------- | --------------------------------------------------------------- | ------------------------------------- |
| `save`     | `(&self, name: &str, data: &GameSystemFile) -> Result<PathBuf>` | Save to base dir, return written path |
| `save_at`  | `(&self, path: &Path, data: &GameSystemFile) -> Result<()>`     | Save to specific path (Save As)       |
| `load`     | `(&self, path: &Path) -> Result<GameSystemFile>`                | Load from specific path               |
| `list`     | `(&self) -> Result<Vec<ProjectEntry>>`                          | List `.hexorder` files in base dir    |
| `delete`   | `(&self, path: &Path) -> Result<()>`                            | Delete a saved file                   |
| `base_dir` | `(&self) -> &Path`                                              | The resolved base directory           |

All `Result` types use `PersistenceError` from the persistence contract.

### `Storage`

Bevy resource wrapping a boxed `StorageProvider`.

| Method     | Signature                                      | Description                    |
| ---------- | ---------------------------------------------- | ------------------------------ |
| `new`      | `(provider: Box<dyn StorageProvider>) -> Self` | Create from a provider         |
| `provider` | `(&self) -> &dyn StorageProvider`              | Access the underlying provider |

Manual `Debug` impl (cannot derive for trait objects).

## Save Directory Resolution

Default directory for the file dialog on first save. Resolution order (compile-time feature flags):

1. `macos` feature: `~/Library/Application Support/hexorder/`
2. `xdg` feature: `$XDG_DATA_HOME/hexorder/` (or `~/.local/share/hexorder/`)
3. Default: `.state/{cargo-pkg-version}/` relative to `CARGO_MANIFEST_DIR`

Features `xdg` and `macos` are mutually exclusive (compile error if both enabled).

The directory is created on demand when the first save occurs.

CLI callers can insert a `StorageConfig` resource before adding `PersistencePlugin` to override the
default resolution.

## Consumed By

- `persistence` plugin — `FilesystemProvider` implementation, save/load systems
- `editor_ui` plugin — (future) launcher project list via `list()`

## Dependencies

- `persistence` contract — `GameSystemFile`, `PersistenceError`
