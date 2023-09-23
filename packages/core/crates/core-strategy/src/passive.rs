use crate::strategy::SingularStrategy;
use crate::Strategies;
use std::fmt::{Display, Formatter};

/// This strategy does nothing.
pub struct PassiveStrategy;

impl PassiveStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

impl Display for PassiveStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategies::Passive)
    }
}

impl SingularStrategy for PassiveStrategy {}
