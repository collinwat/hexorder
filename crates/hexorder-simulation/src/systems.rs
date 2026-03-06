use bevy::prelude::*;

use crate::events::{DieRolled, TableResolved};

/// Observer: log die roll events (future: update UI).
pub fn on_die_rolled(_trigger: On<DieRolled>) {
    // Future: notify editor UI, update roll display panel.
}

/// Observer: log table resolution events (future: update UI).
pub fn on_table_resolved(_trigger: On<TableResolved>) {
    // Future: notify editor UI, apply outcome effects.
}
