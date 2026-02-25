//! Re-export shim: all contract types now live in the `hexorder-contracts` crate.
//! Plugin code can continue using `crate::contracts::*` until imports are updated
//! to `hexorder_contracts::*` (Scope 2).

pub use hexorder_contracts::*;
