//! Plugin-local components and resources for `editor_ui`.
//!
//! Contract types (`EditorTool`) live in `crate::contracts::editor_ui`.
//! This module holds types that are internal to the `editor_ui` plugin.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;
use egui_dock::DockState;

/// Brand palette constants for the editor UI.
/// Source of truth: `docs/brand.md`
pub(crate) struct BrandTheme;

impl BrandTheme {
    // -- Backgrounds --
    /// Deep background (#0a0a0a) — deepest UI panels
    pub const BG_DEEP: egui::Color32 = egui::Color32::from_gray(10);
    /// Panel fill (#191919) — panel backgrounds
    pub const BG_PANEL: egui::Color32 = egui::Color32::from_gray(25);
    /// Surface (#232323) — interactive surface areas / faint bg
    pub const BG_SURFACE: egui::Color32 = egui::Color32::from_gray(35);

    // -- Widget fills (graduated brightness for state) --
    pub const WIDGET_NONINTERACTIVE: egui::Color32 = egui::Color32::from_gray(30);
    pub const WIDGET_INACTIVE: egui::Color32 = egui::Color32::from_gray(40);
    pub const WIDGET_HOVERED: egui::Color32 = egui::Color32::from_gray(55);
    pub const WIDGET_ACTIVE: egui::Color32 = egui::Color32::from_gray(70);

    // -- Accent --
    /// Teal (#005c80) — selection highlights, active states
    pub const ACCENT_TEAL: egui::Color32 = egui::Color32::from_rgb(0, 92, 128);
    /// Amber/gold (#c89640) — emphasis, headings, primary actions
    pub const ACCENT_AMBER: egui::Color32 = egui::Color32::from_rgb(200, 150, 64);

    // -- Text --
    /// Primary text (#e0e0e0) — body text, labels
    pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_gray(224);
    /// Secondary text (#808080) — secondary labels, hints
    pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_gray(128);
    /// Disabled text (#505050) — inactive elements
    #[allow(dead_code)]
    pub const TEXT_DISABLED: egui::Color32 = egui::Color32::from_gray(80);
    /// Tertiary text — used for IDs, de-emphasized metadata
    pub const TEXT_TERTIARY: egui::Color32 = egui::Color32::from_gray(120);

    // -- Border --
    /// Subtle border (#3c3c3c) — panel borders, dividers
    pub const BORDER_SUBTLE: egui::Color32 = egui::Color32::from_gray(60);

    // -- Semantic --
    /// Danger (#c85050) — destructive actions, error states
    pub const DANGER: egui::Color32 = egui::Color32::from_rgb(200, 80, 80);
    /// Success (#509850) — valid states, confirmations
    pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(80, 152, 80);
}

use crate::contracts::game_system::{
    ActiveBoardType, ActiveTokenType, EntityRole, GameSystem, PropertyType, SelectedUnit, TypeId,
};
use crate::contracts::hex_grid::SelectedHex;
use crate::contracts::mechanics::{
    CombatModifierRegistry, CombatResultsTable, CrtColumnType, ModifierSource, PhaseType,
    TurnStructure,
};
use crate::contracts::ontology::{
    ConceptRegistry, ConstraintExpr, ConstraintRegistry, RelationEffect, RelationRegistry,
    RelationTrigger,
};
use crate::contracts::persistence::Workspace;
use crate::contracts::validation::SchemaValidation;

/// Deferred actions to apply after the egui closure completes.
/// Avoids side effects inside the closure (multi-pass safe).
#[derive(Debug)]
pub(crate) enum EditorAction {
    CreateEntityType {
        name: String,
        role: EntityRole,
        color: Color,
    },
    DeleteEntityType {
        id: TypeId,
    },
    AddProperty {
        type_id: TypeId,
        name: String,
        prop_type: PropertyType,
        enum_options: String,
    },
    RemoveProperty {
        type_id: TypeId,
        prop_id: TypeId,
    },
    DeleteSelectedUnit,
    CreateConcept {
        name: String,
        description: String,
    },
    DeleteConcept {
        id: TypeId,
    },
    AddConceptRole {
        concept_id: TypeId,
        name: String,
        allowed_roles: Vec<EntityRole>,
    },
    RemoveConceptRole {
        concept_id: TypeId,
        role_id: TypeId,
    },
    BindEntityToConcept {
        entity_type_id: TypeId,
        concept_id: TypeId,
        concept_role_id: TypeId,
    },
    UnbindEntityFromConcept {
        #[allow(dead_code)]
        concept_id: TypeId,
        binding_id: TypeId,
    },
    CreateRelation {
        name: String,
        concept_id: TypeId,
        subject_role_id: TypeId,
        object_role_id: TypeId,
        trigger: RelationTrigger,
        effect: RelationEffect,
    },
    DeleteRelation {
        id: TypeId,
    },
    CreateConstraint {
        name: String,
        description: String,
        concept_id: TypeId,
        expression: ConstraintExpr,
    },
    DeleteConstraint {
        id: TypeId,
    },
    CreateEnum {
        name: String,
        options: Vec<String>,
    },
    DeleteEnum {
        id: TypeId,
    },
    AddEnumOption {
        enum_id: TypeId,
        option: String,
    },
    RemoveEnumOption {
        enum_id: TypeId,
        option: String,
    },
    CreateStruct {
        name: String,
    },
    DeleteStruct {
        id: TypeId,
    },
    AddStructField {
        struct_id: TypeId,
        name: String,
        prop_type: PropertyType,
    },
    RemoveStructField {
        struct_id: TypeId,
        field_id: TypeId,
    },
    // -- Mechanics --
    SetPlayerOrder {
        order: crate::contracts::mechanics::PlayerOrder,
    },
    AddPhase {
        name: String,
        phase_type: PhaseType,
    },
    RemovePhase {
        id: TypeId,
    },
    MovePhaseUp {
        id: TypeId,
    },
    MovePhaseDown {
        id: TypeId,
    },
    AddCrtColumn {
        label: String,
        column_type: CrtColumnType,
        threshold: f64,
    },
    RemoveCrtColumn {
        index: usize,
    },
    AddCrtRow {
        label: String,
        die_min: u32,
        die_max: u32,
    },
    RemoveCrtRow {
        index: usize,
    },
    SetCrtOutcome {
        row: usize,
        col: usize,
        label: String,
    },
    AddCombatModifier {
        name: String,
        source: ModifierSource,
        shift: i32,
        priority: i32,
    },
    RemoveCombatModifier {
        id: TypeId,
    },
}

/// Which tab is active in the ontology editor panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OntologyTab {
    #[default]
    Types,
    Enums,
    Structs,
    Concepts,
    Relations,
    Constraints,
    Validation,
    Mechanics,
}

/// Persistent UI state for the editor panels.
#[allow(clippy::struct_excessive_bools)]
#[derive(Resource, Debug)]
pub struct EditorState {
    /// Whether the inspector panel (tile/unit details) is visible.
    pub inspector_visible: bool,
    /// Whether the toolbar (tool mode selector) is visible.
    pub toolbar_visible: bool,
    /// Whether the debug inspector panel (right side) is visible.
    /// Only meaningful when compiled with the `inspector` feature.
    pub debug_panel_visible: bool,
    /// Name for a new entity type being created.
    pub new_type_name: String,
    /// Color for a new entity type (RGB, 0.0-1.0).
    pub new_type_color: [f32; 3],
    /// Selected role index for new entity type (0 = `BoardPosition`, 1 = `Token`).
    /// Currently the role is determined by which section ("Cell Types" / "Unit Types")
    /// the user creates a type in. This field is reserved for a future unified creation panel.
    #[allow(dead_code)]
    pub new_type_role_index: usize,
    /// Name for a new property being added to an entity type.
    pub new_prop_name: String,
    /// Selected property type index (0=Bool, 1=Int, 2=Float, 3=String, 4=Color, 5=Enum,
    /// 6=EntityRef, 7=List, 8=Map, 9=Struct, 10=IntRange, 11=FloatRange).
    pub new_prop_type_index: usize,
    /// Comma-separated enum options when adding an Enum property.
    pub new_enum_options: String,
    /// Role filter for `EntityRef`: 0=Any, 1=`BoardPosition`, 2=Token.
    pub new_prop_entity_ref_role: usize,
    /// Inner type index for List properties (indexes into base types).
    pub new_prop_list_inner_type: usize,
    /// Enum key ID for Map properties.
    pub new_prop_map_enum_id: Option<TypeId>,
    /// Value type index for Map properties.
    pub new_prop_map_value_type: usize,
    /// Struct ID for Struct properties.
    pub new_prop_struct_id: Option<TypeId>,
    /// Min for `IntRange` properties.
    pub new_prop_int_range_min: i64,
    /// Max for `IntRange` properties.
    pub new_prop_int_range_max: i64,
    /// Min for `FloatRange` properties.
    pub new_prop_float_range_min: f64,
    /// Max for `FloatRange` properties.
    pub new_prop_float_range_max: f64,

    // Enum editor
    pub new_enum_name: String,
    pub new_enum_option_text: String,
    // Struct editor
    pub new_struct_name: String,
    pub new_struct_field_name: String,
    pub new_struct_field_type_index: usize,

    // -- Ontology tab state --
    /// Which ontology tab is active.
    pub active_tab: OntologyTab,

    // Concept editor
    pub new_concept_name: String,
    pub new_concept_description: String,
    pub new_role_name: String,
    /// Toggles for allowed entity roles: \[`BoardPosition`, `Token`\].
    pub new_role_allowed_roles: Vec<bool>,
    #[allow(dead_code)]
    pub editing_concept_id: Option<TypeId>,

    // Concept binding
    pub binding_entity_type_id: Option<TypeId>,
    pub binding_concept_role_id: Option<TypeId>,

    // Relation editor
    pub new_relation_name: String,
    pub new_relation_concept_index: usize,
    pub new_relation_subject_index: usize,
    pub new_relation_object_index: usize,
    /// 0=OnEnter, 1=OnExit, 2=WhilePresent.
    pub new_relation_trigger_index: usize,
    /// 0=ModifyProperty, 1=Block, 2=Allow.
    pub new_relation_effect_index: usize,
    pub new_relation_target_prop: String,
    pub new_relation_source_prop: String,
    /// 0=Add, 1=Subtract, 2=Multiply, 3=Min, 4=Max.
    pub new_relation_operation_index: usize,

    // Launcher state
    /// Whether the new project name input is visible on the launcher.
    pub launcher_name_input_visible: bool,
    /// Text content of the new project name input.
    pub launcher_project_name: String,
    /// Whether to request focus on the name input next frame.
    pub launcher_request_focus: bool,

    // -- Mechanics tab state --
    pub new_phase_name: String,
    /// 0=Movement, 1=Combat, 2=Admin.
    pub new_phase_type_index: usize,
    pub new_crt_col_label: String,
    /// 0=OddsRatio, 1=Differential.
    pub new_crt_col_type_index: usize,
    pub new_crt_col_threshold: String,
    pub new_crt_row_label: String,
    pub new_crt_row_die_min: String,
    pub new_crt_row_die_max: String,
    pub new_modifier_name: String,
    /// 0=DefenderTerrain, 1=AttackerTerrain, 2=Custom.
    pub new_modifier_source_index: usize,
    pub new_modifier_custom_source: String,
    pub new_modifier_shift: i32,
    pub new_modifier_priority: i32,
    /// Mutable edit buffer for CRT outcome labels, indexed \[row\]\[col\].
    /// Re-synced from `CombatResultsTable` when dimensions change.
    pub crt_outcome_labels: Vec<Vec<String>>,

    // Constraint editor
    pub new_constraint_name: String,
    pub new_constraint_description: String,
    pub new_constraint_concept_index: usize,
    /// 0=PropertyCompare, 1=CrossCompare, 2=IsType, 3=PathBudget.
    pub new_constraint_expr_type_index: usize,
    pub new_constraint_role_index: usize,
    pub new_constraint_property: String,
    /// 0=Eq, 1=Ne, 2=Lt, 3=Le, 4=Gt, 5=Ge.
    pub new_constraint_op_index: usize,
    pub new_constraint_value_str: String,

    // -- Combat panel state (Play mode) --
    pub combat_attacker_strength: f64,
    pub combat_defender_strength: f64,

    // -- Settings --
    /// Base font size in points. Range 10.0–24.0, default 15.0.
    pub font_size_base: f32,

    // -- About panel --
    /// Whether the About panel is visible.
    pub about_panel_visible: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            inspector_visible: true,
            toolbar_visible: true,
            debug_panel_visible: false,
            new_type_name: String::new(),
            new_type_color: [0.5, 0.5, 0.5],
            new_type_role_index: 0,
            new_prop_name: String::new(),
            new_prop_type_index: 0,
            new_enum_options: String::new(),
            new_prop_entity_ref_role: 0,
            new_prop_list_inner_type: 0,
            new_prop_map_enum_id: None,
            new_prop_map_value_type: 0,
            new_prop_struct_id: None,
            new_prop_int_range_min: 0,
            new_prop_int_range_max: 100,
            new_prop_float_range_min: 0.0,
            new_prop_float_range_max: 1.0,
            new_enum_name: String::new(),
            new_enum_option_text: String::new(),
            new_struct_name: String::new(),
            new_struct_field_name: String::new(),
            new_struct_field_type_index: 0,
            active_tab: OntologyTab::default(),
            launcher_name_input_visible: false,
            launcher_project_name: String::new(),
            launcher_request_focus: false,
            new_concept_name: String::new(),
            new_concept_description: String::new(),
            new_role_name: String::new(),
            new_role_allowed_roles: vec![false, false],
            editing_concept_id: None,
            binding_entity_type_id: None,
            binding_concept_role_id: None,
            new_relation_name: String::new(),
            new_relation_concept_index: 0,
            new_relation_subject_index: 0,
            new_relation_object_index: 0,
            new_relation_trigger_index: 0,
            new_relation_effect_index: 0,
            new_relation_target_prop: String::new(),
            new_relation_source_prop: String::new(),
            new_relation_operation_index: 0,
            new_phase_name: String::new(),
            new_phase_type_index: 0,
            new_crt_col_label: String::new(),
            new_crt_col_type_index: 0,
            new_crt_col_threshold: String::new(),
            new_crt_row_label: String::new(),
            new_crt_row_die_min: String::new(),
            new_crt_row_die_max: String::new(),
            new_modifier_name: String::new(),
            new_modifier_source_index: 0,
            new_modifier_custom_source: String::new(),
            new_modifier_shift: 0,
            new_modifier_priority: 0,
            crt_outcome_labels: Vec::new(),
            new_constraint_name: String::new(),
            new_constraint_description: String::new(),
            new_constraint_concept_index: 0,
            new_constraint_expr_type_index: 0,
            new_constraint_role_index: 0,
            new_constraint_property: String::new(),
            new_constraint_op_index: 0,
            new_constraint_value_str: String::new(),
            combat_attacker_strength: 0.0,
            combat_defender_strength: 0.0,
            font_size_base: 15.0,
            about_panel_visible: false,
        }
    }
}

/// Bundled system parameter for project-level read-only resources.
/// Reduces the system parameter count in `editor_tool_palette_system`.
#[derive(SystemParam)]
pub(super) struct ProjectParams<'w> {
    pub(super) workspace: Res<'w, Workspace>,
    pub(super) game_system: Res<'w, GameSystem>,
}

/// Bundled system parameter for active selection state.
/// Reduces the system parameter count in `editor_tool_palette_system`.
#[derive(SystemParam)]
pub(super) struct SelectionParams<'w> {
    pub(super) active_board: ResMut<'w, ActiveBoardType>,
    pub(super) active_token: ResMut<'w, ActiveTokenType>,
    pub(super) selected_unit: ResMut<'w, SelectedUnit>,
    pub(super) multi: Res<'w, crate::contracts::editor_ui::Selection>,
    /// Used by the Inspector tab (Scope 3 — Query migration).
    #[allow(dead_code)]
    pub(super) selected_hex: Res<'w, SelectedHex>,
}

/// Bundled system parameter for mechanics-related resources.
/// Reduces the system parameter count in `editor_tool_palette_system`.
#[derive(SystemParam)]
pub(super) struct MechanicsParams<'w> {
    pub(super) turn_structure: ResMut<'w, TurnStructure>,
    pub(super) combat_results_table: ResMut<'w, CombatResultsTable>,
    pub(super) combat_modifiers: ResMut<'w, CombatModifierRegistry>,
}

/// Bundled system parameter for ontology-related resources.
/// Reduces the system parameter count in `editor_tool_palette_system`.
#[derive(SystemParam)]
pub(super) struct OntologyParams<'w> {
    pub(super) concept_registry: ResMut<'w, ConceptRegistry>,
    pub(super) relation_registry: ResMut<'w, RelationRegistry>,
    pub(super) constraint_registry: ResMut<'w, ConstraintRegistry>,
    pub(super) schema_validation: Res<'w, SchemaValidation>,
}

/// Whether the grid coordinate overlay is visible. Toggled by G key.
#[derive(Resource, Debug, Default)]
pub(crate) struct GridOverlayVisible(pub(crate) bool);

/// State for the toast notification system. Single-slot, no stacking.
#[derive(Resource, Debug, Default)]
pub(crate) struct ToastState {
    pub(crate) active: Option<ActiveToast>,
}

/// An active toast being displayed.
#[derive(Debug, Clone)]
pub(crate) struct ActiveToast {
    pub(crate) message: String,
    pub(crate) kind: crate::contracts::editor_ui::ToastKind,
    /// Remaining time in seconds before the toast disappears.
    pub(crate) remaining: f32,
}

// ---------------------------------------------------------------------------
// Dock layout (Scope 1 — egui_dock evaluation)
// ---------------------------------------------------------------------------

/// Which logical panel occupies a dock tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DockTab {
    /// 3D scene — always present, transparent background.
    Viewport,
    /// Tool mode + cell/unit palette (left zone).
    ToolPalette,
    /// Tile/unit inspector (right zone).
    Inspector,
    /// Validation output (bottom zone).
    Validation,
}

impl std::fmt::Display for DockTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Viewport => write!(f, "Viewport"),
            Self::ToolPalette => write!(f, "Tool Palette"),
            Self::Inspector => write!(f, "Inspector"),
            Self::Validation => write!(f, "Validation"),
        }
    }
}

/// Persistent dock layout state wrapping `egui_dock::DockState`.
/// Currently used by tests; active rendering moves here in Scope 4 (tab support).
#[derive(Resource)]
pub(crate) struct DockLayoutState {
    #[allow(dead_code)]
    pub(crate) dock_state: DockState<DockTab>,
}

impl std::fmt::Debug for DockLayoutState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DockLayoutState")
            .field("dock_state", &"<DockState>")
            .finish()
    }
}

impl Default for DockLayoutState {
    fn default() -> Self {
        Self {
            dock_state: create_default_dock_layout(),
        }
    }
}

/// Creates the default four-zone dock layout.
///
/// Layout: Left (20%) | Center viewport (~60%) | Right (~20%) | Bottom (~12%)
pub(crate) fn create_default_dock_layout() -> DockState<DockTab> {
    let mut state = DockState::new(vec![DockTab::Viewport]);
    let tree = state.main_surface_mut();
    let root = egui_dock::NodeIndex::root();

    // Left: ToolPalette gets 20% width.
    let [center, _left] = tree.split_left(root, 0.20, vec![DockTab::ToolPalette]);

    // Right: Inspector gets 25% of remaining width (after left split).
    let [center, _right] = tree.split_right(center, 0.75, vec![DockTab::Inspector]);

    // Bottom: Validation gets 15% of center height.
    let [_viewport, _bottom] = tree.split_below(center, 0.85, vec![DockTab::Validation]);

    state
}
