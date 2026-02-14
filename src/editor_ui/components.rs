//! Plugin-local components and resources for `editor_ui`.
//!
//! Contract types (`EditorTool`) live in `crate::contracts::editor_ui`.
//! This module holds types that are internal to the `editor_ui` plugin.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::contracts::game_system::{EntityRole, PropertyType, TypeId};
use crate::contracts::ontology::{
    ConceptRegistry, ConstraintExpr, ConstraintRegistry, RelationEffect, RelationRegistry,
    RelationTrigger,
};
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
}

/// Which tab is active in the ontology editor panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OntologyTab {
    #[default]
    Types,
    Concepts,
    Relations,
    Constraints,
    Validation,
}

/// Persistent UI state for the editor panels.
#[derive(Resource, Debug)]
pub struct EditorState {
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
    /// Selected property type index (0=Bool, 1=Int, 2=Float, 3=String, 4=Color, 5=Enum).
    pub new_prop_type_index: usize,
    /// Comma-separated enum options when adding an Enum property.
    pub new_enum_options: String,

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
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            new_type_name: String::new(),
            new_type_color: [0.5, 0.5, 0.5],
            new_type_role_index: 0,
            new_prop_name: String::new(),
            new_prop_type_index: 0,
            new_enum_options: String::new(),
            active_tab: OntologyTab::default(),
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
            new_constraint_name: String::new(),
            new_constraint_description: String::new(),
            new_constraint_concept_index: 0,
            new_constraint_expr_type_index: 0,
            new_constraint_role_index: 0,
            new_constraint_property: String::new(),
            new_constraint_op_index: 0,
            new_constraint_value_str: String::new(),
        }
    }
}

/// Bundled system parameter for ontology-related resources.
/// Reduces the system parameter count in `editor_panel_system`.
#[derive(SystemParam)]
pub(super) struct OntologyParams<'w> {
    pub(super) concept_registry: ResMut<'w, ConceptRegistry>,
    pub(super) relation_registry: ResMut<'w, RelationRegistry>,
    pub(super) constraint_registry: ResMut<'w, ConstraintRegistry>,
    pub(super) schema_validation: Res<'w, SchemaValidation>,
}
