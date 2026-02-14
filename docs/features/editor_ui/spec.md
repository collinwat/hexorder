# Feature: editor_ui

## Summary

Provides the egui-based editor interface. Evolves through releases: 0.1.0 minimal toolbar, 0.2.0
dark theme + cell type editor + inspector, 0.3.0 unit palette + unit type editor + unit inspector,
0.4.0 unified entity type editor + ontology panels (concepts, relations, constraints, validation).

## Plugin

- Module: `src/editor_ui/`
- Plugin struct: `EditorUiPlugin`
- Schedule: `EguiPrimaryContextPass` (UI rendering via bevy_egui)

## Dependencies

- **Contracts consumed**: `hex_grid` (HexPosition, SelectedHex, HexTile), `game_system` (GameSystem,
  EntityType, EntityRole, EntityTypeRegistry, EntityData, ActiveBoardType, ActiveTokenType,
  SelectedUnit, UnitInstance, TypeId, PropertyDefinition, PropertyType, PropertyValue,
  EnumDefinition), `editor_ui` (EditorTool, PaintPreview), `ontology` (ConceptRegistry,
  RelationRegistry, ConstraintRegistry, Concept, ConceptRole, ConceptBinding, PropertyBinding,
  Relation, RelationTrigger, RelationEffect, ModifyOperation, Constraint, ConstraintExpr,
  CompareOp), `validation` (SchemaValidation, SchemaError, ValidMoveSet, ValidationResult)
- **Contracts produced**: `editor_ui` (EditorTool, PaintPreview)
- **Crate dependencies**: `bevy_egui` (see `docs/guides/bevy-egui.md`)

## Requirements

### 0.1.0 (retained)

1. [REQ-MODE] Maintain an `EditorTool` resource with modes: Select, Paint, Place.
2. [REQ-TOOLBAR] Render an egui left side panel with a tool mode selector.
3. [REQ-NO-PASSTHROUGH] When the mouse is over an egui panel, input does not pass through.

### 0.2.0 (retained)

4. [REQ-DARK-THEME] Apply a dark color scheme to all egui panels.
5. [REQ-GAME-SYSTEM-INFO] Display the Game System id (abbreviated) and version.

### 0.4.0 (new — unified entity editor + ontology panels)

6. [REQ-TABBED-LAYOUT] The left sidebar uses a tabbed layout with tabs: Types, Concepts, Relations,
   Constraints, Validation. Context-sensitive palette (Paint/Place) and inspector remain below the
   tabs.
7. [REQ-UNIFIED-TYPE-EDITOR] The Types tab shows a single entity type editor replacing the separate
   cell/unit type editors. Features:
    - Role selector (BoardPosition / Token) on type creation
    - List of all entity types, filterable by role
    - Name, color, property editing (same widgets as 0.3.0)
    - Concept binding summary: "Participates in: Motion (as traveler)"
8. [REQ-ENTITY-PALETTE] In Paint mode, show entity types filtered by BoardPosition. In Place mode,
   show entity types filtered by Token. Same swatch+name layout as 0.3.0.
9. [REQ-CONCEPT-EDITOR] The Concepts tab provides UI for:
    - Creating/editing/deleting concepts (name, description)
    - Adding/removing role slots on a concept (name, allowed entity roles)
    - Binding entity types to concept roles with property mappings
    - Viewing which entity types are bound to each role
10. [REQ-RELATION-EDITOR] The Relations tab provides UI for:
    - Creating/editing/deleting relations
    - Selecting concept, subject role, object role
    - Selecting trigger (OnEnter, OnExit, WhilePresent)
    - Selecting effect type (ModifyProperty, Block, Allow) and configuring parameters
    - Showing auto-generated constraint preview
11. [REQ-CONSTRAINT-EDITOR] The Constraints tab provides UI for:
    - Viewing all constraints (auto-generated marked with "[auto]" badge)
    - Creating manual constraints
    - Editing constraint expressions via structured form:
        - Dropdown for expression type (PropertyCompare, CrossCompare, IsType, PathBudget)
        - Fields for role, property, operator, value
        - PathBudget as a dedicated widget
    - Deleting constraints (including auto-generated ones)
12. [REQ-VALIDATION-PANEL] The Validation tab shows:
    - Overall schema validity (green checkmark or red X)
    - List of schema errors with category, message, and source reference
13. [REQ-INSPECTOR-EVOLUTION] The inspector panel (shown when a tile or unit is selected) gains:
    - Concept binding annotations: "Movement Points: 4 (Motion budget)"
    - Valid move count when a unit is selected: "Can reach N positions"
    - When hovering a blocked hex: constraint violation details

### Deferred Action Pattern

The existing EditorAction enum extends with new variants for ontology operations:

- CreateEntityType, DeleteEntityType (unified, with role)
- CreateConcept, DeleteConcept, AddConceptRole, RemoveConceptRole
- BindEntityToConcept, UnbindEntityFromConcept
- CreateRelation, DeleteRelation
- CreateConstraint, DeleteConstraint, UpdateConstraint

## Success Criteria

### 0.1.0–0.3.0 (retained)

- [x] [SC-1] Tool mode switches between Select, Paint, and Place
- [x] [SC-5] Clicking on UI panels does not trigger hex tile selection
- [x] [SC-6] Editor uses a dark theme
- [x] [SC-13] Game System id and version are displayed

### 0.4.0 (new)

- [x] [SC-14] Tabbed layout renders with 5 tabs: Types, Concepts, Relations, Constraints, Validation
- [x] [SC-15] Unified type editor shows entity types from both roles
- [x] [SC-16] Type creation includes role selector (BoardPosition / Token)
- [x] [SC-17] Concept editor can create a concept with role slots
- [x] [SC-18] Concept editor can bind an entity type to a concept role
- [x] [SC-19] Relation editor can create a relation between concept roles
- [x] [SC-20] Constraint editor can create a PropertyCompare constraint
- [x] [SC-21] Constraint editor can create a PathBudget constraint
- [x] [SC-22] Auto-generated constraints show "[auto]" badge
- [x] [SC-23] Validation panel shows schema errors when ontology is invalid
- [x] [SC-24] Entity palette filters by role (BoardPosition in Paint, Token in Place)
- [x] [SC-BUILD] `cargo build` succeeds
- [x] [SC-CLIPPY] `cargo clippy --all-targets` passes
- [x] [SC-TEST] `cargo test` passes
- [x] [SC-BOUNDARY] No imports from other features' internals

## Constraints

- The dark theme must use the brand palette from `docs/brand.md`
- New color literals must be added to the approved palette in the architecture test
- Property editors must validate input
- The tabbed layout should not overwhelm the designer — collapse sections by default, show
  contextually relevant content
- The constraint expression builder is structured (dropdowns + fields), not free text

## Open Questions

- None
