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
}

impl MechanicCatalog {
    /// Returns all entries in the given category.
    pub fn entries_by_category(&self, category: MechanicCategory) -> Vec<&MechanicEntry> {
        self.entries
            .iter()
            .filter(|e| e.category == category)
            .collect()
    }

    /// Returns all entries that have a scaffolding template available.
    pub fn entries_with_templates(&self) -> Vec<&MechanicEntry> {
        self.entries
            .iter()
            .filter(|e| matches!(e.template, TemplateAvailability::Available { .. }))
            .collect()
    }
}
