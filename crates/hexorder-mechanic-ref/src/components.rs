//! Plugin-local re-exports of contract types.
//!
//! All shared types live in `hexorder_contracts::mechanic_reference`.
//! This module re-exports them for plugin-internal convenience.

pub use hexorder_contracts::mechanic_reference::{
    MechanicCatalog, MechanicCategory, MechanicEntry, ScaffoldAction, ScaffoldRecipe,
    TemplateAvailability,
};
