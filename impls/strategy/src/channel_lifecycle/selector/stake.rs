//! Per-peer on-chain safe-balance score.

use std::collections::HashMap;

use hopr_api::types::primitive::prelude::Address;

/// Per-peer normalized stake score in [0, 1], populated only when the active
/// selector requests the `STAKE` signal.  Peers not present score `0.0`.
pub struct StakeView {
    scores: HashMap<Address, f64>,
}

impl StakeView {
    pub fn empty() -> Self {
        Self { scores: HashMap::new() }
    }

    pub fn from_scores(scores: HashMap<Address, f64>) -> Self {
        Self { scores }
    }

    /// Returns the normalized stake score for `addr`, or `0.0` if unknown.
    pub fn score(&self, addr: &Address) -> f64 {
        self.scores.get(addr).copied().unwrap_or(0.0)
    }
}
