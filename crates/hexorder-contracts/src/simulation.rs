#![allow(clippy::used_underscore_binding)]
//! Shared simulation types. See `docs/contracts/simulation.md`.
//!
//! Defines seeded RNG, die types, roll logging, lookup tables, and
//! resolution tables. These are generic simulation primitives per ADR-005.

use bevy::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Seeded RNG
// ---------------------------------------------------------------------------

/// The type of die being rolled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum DieType {
    /// Standard 6-sided die (1-6).
    D6,
    /// 10-sided die (1-10).
    D10,
    /// Percentile die (1-100).
    D100,
    /// Arbitrary die with the given number of sides (1-sides).
    Custom { sides: u32 },
}

impl DieType {
    /// Returns the number of sides for this die type.
    #[must_use]
    pub fn sides(self) -> u32 {
        match self {
            Self::D6 => 6,
            Self::D10 => 10,
            Self::D100 => 100,
            Self::Custom { sides } => sides,
        }
    }
}

/// A single recorded die roll.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct RollRecord {
    /// Monotonically increasing index within this RNG session.
    pub roll_index: u64,
    /// What kind of die was rolled.
    pub die_type: DieType,
    /// The result value (1-based).
    pub result: u32,
    /// Human-readable context (e.g., "CRT resolution at 3:1").
    pub context: String,
}

/// Deterministic RNG resource wrapping `ChaCha8Rng`.
///
/// All randomness in simulation flows through this resource.
/// The full roll log enables deterministic replay and future
/// Monte Carlo analysis (#57).
#[derive(Resource, Debug)]
pub struct SimulationRng {
    /// The seed used to initialize this RNG.
    seed: u64,
    /// The underlying deterministic RNG.
    rng: ChaCha8Rng,
    /// Complete log of all rolls made with this RNG.
    roll_log: Vec<RollRecord>,
    /// Monotonic counter for roll indexing.
    next_roll_index: u64,
}

impl SimulationRng {
    /// Create a new RNG with the given seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            rng: ChaCha8Rng::seed_from_u64(seed),
            roll_log: Vec::new(),
            next_roll_index: 0,
        }
    }

    /// Create a new RNG with a random seed.
    #[must_use]
    pub fn new_random() -> Self {
        let seed = rand::random::<u64>();
        Self::new(seed)
    }

    /// Returns the seed used to initialize this RNG.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Returns the full roll log.
    #[must_use]
    pub fn roll_log(&self) -> &[RollRecord] {
        &self.roll_log
    }

    /// Returns the number of rolls made so far.
    #[must_use]
    pub fn roll_count(&self) -> u64 {
        self.next_roll_index
    }
}

// ---------------------------------------------------------------------------
// RNG Functions (stubs — implementations in next task)
// ---------------------------------------------------------------------------

/// Roll a die of the given type, logging the result.
/// Returns a value in the range `[1, die.sides()]`.
#[must_use]
pub fn roll_die(rng: &mut SimulationRng, die: DieType, context: &str) -> u32 {
    let sides = die.sides();
    let result = rng.rng.random_range(1..=sides);
    let record = RollRecord {
        roll_index: rng.next_roll_index,
        die_type: die,
        result,
        context: context.to_string(),
    };
    rng.roll_log.push(record);
    rng.next_roll_index += 1;
    result
}

/// Roll a value in the inclusive range `[min, max]`, logging the result.
/// Uses `DieType::Custom` with `max - min + 1` sides.
#[must_use]
pub fn roll_range(rng: &mut SimulationRng, min: u32, max: u32, context: &str) -> u32 {
    let sides = max - min + 1;
    let die = DieType::Custom { sides };
    let raw = roll_die(rng, die, context);
    // roll_die returns [1, sides], shift to [min, max]
    raw - 1 + min
}

/// Reset the RNG with a new seed, clearing the roll log.
pub fn reset_rng(rng: &mut SimulationRng, seed: u64) {
    rng.seed = seed;
    rng.rng = ChaCha8Rng::seed_from_u64(seed);
    rng.roll_log.clear();
    rng.next_roll_index = 0;
}

/// Replay rolls from a seed — returns the first `count` d6 results.
/// Useful for verifying deterministic replay.
#[must_use]
pub fn replay_from_seed(seed: u64, count: u64) -> Vec<u32> {
    let mut rng = SimulationRng::new(seed);
    (0..count)
        .map(|_| roll_die(&mut rng, DieType::D6, ""))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_type_sides() {
        assert_eq!(DieType::D6.sides(), 6);
        assert_eq!(DieType::D10.sides(), 10);
        assert_eq!(DieType::D100.sides(), 100);
        assert_eq!(DieType::Custom { sides: 20 }.sides(), 20);
    }

    #[test]
    fn new_rng_has_empty_log() {
        let rng = SimulationRng::new(42);
        assert_eq!(rng.seed(), 42);
        assert_eq!(rng.roll_count(), 0);
        assert!(rng.roll_log().is_empty());
    }

    #[test]
    fn roll_die_d6_in_range() {
        let mut rng = SimulationRng::new(42);
        for _ in 0..100 {
            let result = roll_die(&mut rng, DieType::D6, "test");
            assert!((1..=6).contains(&result), "d6 result {result} out of range");
        }
    }

    #[test]
    fn roll_die_logs_result() {
        let mut rng = SimulationRng::new(42);
        let result = roll_die(&mut rng, DieType::D6, "combat");
        assert_eq!(rng.roll_count(), 1);
        assert_eq!(rng.roll_log().len(), 1);
        let record = &rng.roll_log()[0];
        assert_eq!(record.roll_index, 0);
        assert_eq!(record.die_type, DieType::D6);
        assert_eq!(record.result, result);
        assert_eq!(record.context, "combat");
    }

    #[test]
    fn roll_die_deterministic() {
        let mut rng1 = SimulationRng::new(42);
        let mut rng2 = SimulationRng::new(42);
        for _ in 0..50 {
            assert_eq!(
                roll_die(&mut rng1, DieType::D6, ""),
                roll_die(&mut rng2, DieType::D6, ""),
            );
        }
    }

    #[test]
    fn roll_range_in_bounds() {
        let mut rng = SimulationRng::new(42);
        for _ in 0..100 {
            let result = roll_range(&mut rng, 3, 8, "test");
            assert!(
                (3..=8).contains(&result),
                "range result {result} out of [3,8]"
            );
        }
    }

    #[test]
    fn reset_rng_clears_log_and_reseeds() {
        let mut rng = SimulationRng::new(42);
        let _ = roll_die(&mut rng, DieType::D6, "before");
        assert_eq!(rng.roll_count(), 1);

        reset_rng(&mut rng, 99);
        assert_eq!(rng.seed(), 99);
        assert_eq!(rng.roll_count(), 0);
        assert!(rng.roll_log().is_empty());
    }

    #[test]
    fn replay_from_seed_matches_sequential_rolls() {
        let replayed = replay_from_seed(42, 10);
        assert_eq!(replayed.len(), 10);

        let mut rng = SimulationRng::new(42);
        for expected in &replayed {
            assert_eq!(roll_die(&mut rng, DieType::D6, ""), *expected);
        }
    }

    #[test]
    fn roll_die_d100_in_range() {
        let mut rng = SimulationRng::new(42);
        for _ in 0..100 {
            let result = roll_die(&mut rng, DieType::D100, "test");
            assert!(
                (1..=100).contains(&result),
                "d100 result {result} out of range"
            );
        }
    }

    #[test]
    fn roll_die_custom_sides() {
        let mut rng = SimulationRng::new(42);
        let die = DieType::Custom { sides: 20 };
        for _ in 0..100 {
            let result = roll_die(&mut rng, die, "test");
            assert!(
                (1..=20).contains(&result),
                "d20 result {result} out of range"
            );
        }
    }

    #[test]
    fn roll_log_index_increments() {
        let mut rng = SimulationRng::new(42);
        let _ = roll_die(&mut rng, DieType::D6, "first");
        let _ = roll_die(&mut rng, DieType::D10, "second");
        let _ = roll_die(&mut rng, DieType::D100, "third");
        assert_eq!(rng.roll_log()[0].roll_index, 0);
        assert_eq!(rng.roll_log()[1].roll_index, 1);
        assert_eq!(rng.roll_log()[2].roll_index, 2);
    }
}
