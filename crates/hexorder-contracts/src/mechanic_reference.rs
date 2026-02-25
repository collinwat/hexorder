//! Shared Mechanic Reference types. See `docs/contracts/mechanic-reference.md`.
//!
//! Read-only catalog of wargame mechanics organized by the Engelstein taxonomy.
//! Consumed by `editor_ui` for display; populated by `mechanic_reference` plugin.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Category taxonomy
// ---------------------------------------------------------------------------

/// The six areas of the Engelstein taxonomy as used in the
/// Hex Wargame Mechanics Survey.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MechanicCategory {
    /// Area 1: Core Universal Mechanics — appear in nearly every hex wargame.
    CoreUniversal,
    /// Area 2: Advanced/Common Mechanics — common but not universal.
    AdvancedCommon,
    /// Area 3: Bespoke/Unusual Mechanics — game-specific or rare.
    BespokeUnusual,
    /// Area 4: Game System Architecture — structural/organizational patterns.
    GameSystemArchitecture,
    /// Area 5: Digital Implementation Considerations — digital-specific concerns.
    DigitalImplementation,
    /// Area 6: Evolution of the Genre — historical context.
    GenreEvolution,
}

impl MechanicCategory {
    /// Returns all six categories in survey order.
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            Self::CoreUniversal,
            Self::AdvancedCommon,
            Self::BespokeUnusual,
            Self::GameSystemArchitecture,
            Self::DigitalImplementation,
            Self::GenreEvolution,
        ]
    }

    /// Human-readable display name for the category.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::CoreUniversal => "Core Universal Mechanics",
            Self::AdvancedCommon => "Advanced/Common Mechanics",
            Self::BespokeUnusual => "Bespoke/Unusual Mechanics",
            Self::GameSystemArchitecture => "Game System Architecture",
            Self::DigitalImplementation => "Digital Implementation Considerations",
            Self::GenreEvolution => "Evolution of the Genre",
        }
    }

    /// Short description of what the category covers.
    #[must_use]
    pub fn description(self) -> &'static str {
        match self {
            Self::CoreUniversal => {
                "Mechanics that appear in nearly every hex-and-counter wargame, forming the irreducible foundation."
            }
            Self::AdvancedCommon => {
                "Mechanics common in many wargames but not universal; add depth without being essential."
            }
            Self::BespokeUnusual => {
                "Game-specific or rare mechanics that appear in specialized titles."
            }
            Self::GameSystemArchitecture => {
                "Structural and organizational patterns for rule systems, scenarios, and orders of battle."
            }
            Self::DigitalImplementation => {
                "Considerations specific to digital implementations of hex wargame mechanics."
            }
            Self::GenreEvolution => {
                "Historical context showing how the genre evolved across four generations."
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Template availability
// ---------------------------------------------------------------------------

/// Whether a mechanic entry has a scaffolding template available.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateAvailability {
    /// No scaffolding template — description-only entry.
    None,
    /// A scaffolding template is available.
    Available {
        /// Identifier for the template function (e.g., `"crt_combat"`).
        template_id: String,
        /// Short preview describing what the template scaffolds.
        preview: String,
    },
}

// ---------------------------------------------------------------------------
// Scaffolding actions
// ---------------------------------------------------------------------------

/// A single scaffolding instruction produced by a mechanic template.
#[derive(Debug, Clone)]
pub enum ScaffoldAction {
    /// Create an entity type (cell or token).
    CreateEntityType {
        name: String,
        role: String,
        color: [f32; 3],
    },
    /// Add a property to an entity type (referenced by name).
    AddProperty {
        entity_name: String,
        prop_name: String,
        prop_type: String,
    },
    /// Create an enum definition.
    CreateEnum { name: String, options: Vec<String> },
    /// Add a CRT column.
    AddCrtColumn {
        label: String,
        column_type: String,
        threshold: f64,
    },
    /// Add a CRT row (die roll range).
    AddCrtRow {
        label: String,
        die_min: u32,
        die_max: u32,
    },
    /// Set a CRT outcome cell.
    SetCrtOutcome {
        row: usize,
        col: usize,
        label: String,
    },
    /// Add a turn phase.
    AddPhase { name: String, phase_type: String },
    /// Add a combat modifier.
    AddCombatModifier {
        name: String,
        source: String,
        shift: i32,
        priority: i32,
    },
}

/// A complete scaffolding recipe produced by a mechanic template.
#[derive(Debug, Clone)]
pub struct ScaffoldRecipe {
    /// Template identifier (matches `TemplateAvailability::Available::template_id`).
    pub template_id: String,
    /// Human-readable description of what the template scaffolds.
    pub description: String,
    /// Ordered list of scaffolding actions to execute.
    pub actions: Vec<ScaffoldAction>,
}

// ---------------------------------------------------------------------------
// Catalog entry and resource
// ---------------------------------------------------------------------------

/// A single entry in the mechanic reference catalog.
#[derive(Debug, Clone)]
pub struct MechanicEntry {
    /// The mechanic's name (e.g., "Combat Resolution Systems").
    pub name: String,
    /// Which taxonomy area this mechanic belongs to.
    pub category: MechanicCategory,
    /// Description of how the mechanic works.
    pub description: String,
    /// Games that use this mechanic.
    pub example_games: Vec<String>,
    /// Key design considerations when implementing this mechanic.
    pub design_considerations: String,
    /// Whether a scaffolding template is available for this mechanic.
    pub template: TemplateAvailability,
}

/// Resource holding the full mechanic reference catalog.
/// Populated at startup; read-only thereafter.
#[derive(Resource, Debug, Default)]
pub struct MechanicCatalog {
    pub entries: Vec<MechanicEntry>,
    pub templates: Vec<ScaffoldRecipe>,
}

impl MechanicCatalog {
    /// Returns all entries in the given category.
    #[must_use]
    pub fn entries_by_category(&self, category: MechanicCategory) -> Vec<&MechanicEntry> {
        self.entries
            .iter()
            .filter(|e| e.category == category)
            .collect()
    }

    /// Returns all entries that have a scaffolding template available.
    #[must_use]
    pub fn entries_with_templates(&self) -> Vec<&MechanicEntry> {
        self.entries
            .iter()
            .filter(|e| matches!(e.template, TemplateAvailability::Available { .. }))
            .collect()
    }

    /// Looks up a scaffolding recipe by template ID.
    #[must_use]
    pub fn get_template(&self, template_id: &str) -> Option<ScaffoldRecipe> {
        self.templates
            .iter()
            .find(|r| r.template_id == template_id)
            .cloned()
    }
}
