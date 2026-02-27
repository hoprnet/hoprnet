use std::sync::Arc;

use hopr_internal_types::errors::PathError;

pub type Result<T> = std::result::Result<T, PathPlannerError>;

/// Errors produced by the path planner and graph-based path selector.
#[derive(thiserror::Error, Debug)]
pub enum PathPlannerError {
    #[error("path error: {0}")]
    Path(#[from] PathError),

    #[error("{0}")]
    Other(#[from] anyhow::Error),

    #[error("surb: {0}")]
    Surb(String),

    #[error("api: {0}")]
    Api(String),

    #[error("cache error: {0}")]
    CacheError(#[from] Arc<Self>),
}
