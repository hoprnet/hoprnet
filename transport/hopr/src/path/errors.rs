use std::sync::Arc;

use hopr_api::types::internal::errors::PathError;

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

impl PathPlannerError {
    /// Returns `true` if this error is a SURB-starvation error, including one wrapped by the
    /// path-cache layer as [`CacheError`](PathPlannerError::CacheError). Callers that retry on
    /// transient SURB exhaustion must use this rather than matching [`Surb`](PathPlannerError::Surb)
    /// directly, otherwise a cache-wrapped SURB error is misclassified as a hard failure.
    pub fn is_surb(&self) -> bool {
        match self {
            PathPlannerError::Surb(_) => true,
            PathPlannerError::CacheError(inner) => inner.is_surb(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_surb_detects_direct_and_cache_wrapped() {
        let direct = PathPlannerError::Surb("no surb".into());
        assert!(direct.is_surb(), "direct Surb must be detected");

        // A SURB error surfaced through the path cache arrives wrapped; the old direct-only match
        // would misclassify this as a hard failure and skip the retry.
        let wrapped = PathPlannerError::CacheError(Arc::new(PathPlannerError::Surb("no surb".into())));
        assert!(wrapped.is_surb(), "cache-wrapped Surb must be detected");

        let nested = PathPlannerError::CacheError(Arc::new(wrapped));
        assert!(nested.is_surb(), "doubly cache-wrapped Surb must be detected");

        let other = PathPlannerError::Api("unrelated".into());
        assert!(
            !other.is_surb(),
            "non-Surb error must not be treated as SURB starvation"
        );
        assert!(
            !PathPlannerError::CacheError(Arc::new(other)).is_surb(),
            "cache-wrapped non-Surb error must not be treated as SURB starvation"
        );
    }
}
