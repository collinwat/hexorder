# Plugin: mechanic_reference

## Summary

Provides a read-only browsable catalog of wargame mechanics organized by the Engelstein taxonomy and
the project's Hex Wargame Mechanics Survey. Includes scaffolding templates that create starter
entity types, properties, and rules for mechanics Hexorder already supports.

## Plugin

- Module: `src/mechanic_reference/`
- Plugin struct: `MechanicReferencePlugin`
- Schedule: Update (catalog data is static; UI rendering via editor_ui integration)

## Appetite

- **Size**: Small Batch (1-2 weeks)
- **Pitch**: #100

## Dependencies

- **Contracts consumed**: `game_system` (EntityType, EntityTypeRegistry, PropertyDefinition,
  PropertyValue, EnumDefinition, EnumRegistry), `mechanics` (TurnStructure, CombatResultsTable,
  CombatModifierRegistry, CrtColumn, CrtRow, CombatOutcome, OutcomeEffect)
- **Contracts produced**: None (read-only catalog + scaffolding that writes to existing registries)
- **Crate dependencies**: None new expected

## Scope

1. [SCOPE-1] Catalog data model — define `MechanicEntry`, `MechanicCategory`, and `MechanicCatalog`
   resource holding all catalog entries
2. [SCOPE-2] Catalog content — populate the catalog with entries from the Hex Wargame Mechanics
   Survey, organized by Engelstein taxonomy categories
3. [SCOPE-3] Browsable UI panel — read-only egui panel showing categories, entries with
   descriptions, example games, and design considerations
4. [SCOPE-4] Scaffolding templates — "Use Template" action for 5-8 supported mechanics (CRT combat,
   ZOC, stacking, movement points, terrain effects, LOS, supply, weather) that scaffold entity
   types, properties, and rules into the current game system
5. [SCOPE-5] Template application — wire scaffolding into game system registries, producing fully
   editable instances with no link back to the template

## Success Criteria

- [ ] [SC-1] `MechanicCatalog` resource loads at startup with categorized entries
- [ ] [SC-2] Catalog contains entries for all 6 Engelstein taxonomy areas
- [ ] [SC-3] Browsable panel renders categories and entries with descriptions
- [ ] [SC-4] "Use Template" scaffolds entity types and properties into registries
- [ ] [SC-5] Scaffolded elements are fully editable (no template link)
- [ ] [SC-BUILD] `cargo build` succeeds with this plugin registered
- [ ] [SC-CLIPPY] `cargo clippy -- -D warnings` passes
- [ ] [SC-TEST] `cargo test` passes (all tests, not just this plugin's)
- [ ] [SC-BOUNDARY] No imports from other plugins' internals — all cross-plugin types come from
      `crate::contracts::`

## UAT Checklist

- [ ] [UAT-1] Launch app, open the mechanic reference panel, browse categories and entries
- [ ] [UAT-2] Select a mechanic entry with a template, click "Use Template", verify new entity types
      and properties appear in the game system
- [ ] [UAT-3] Modify a scaffolded entity type (rename, change property) — verify it is fully
      editable with no template connection

## Decomposition (for agent teams)

Solo — fewer than 3 independent subsystems.

## Constraints

- Catalog content is read-only; templates produce one-shot scaffolding with no link back
- Templates only cover mechanics Hexorder already supports (5-8 mechanics)
- No community-contributed mechanics or template marketplace

## Open Questions

- Where does the panel live in the dock system? New DockTab variant, or floating window?
- Should the panel be accessible from both Launcher and Editor states, or Editor only?

## Deferred Items

- No community-contributed mechanics (pitch No Go)
- No template marketplace or sharing (pitch No Go)
- No auto-detection of which mechanics a game system uses (pitch No Go)
- No template dependencies (pitch No Go)
- No variant templates for the same mechanic (pitch No Go)
