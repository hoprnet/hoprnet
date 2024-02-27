#[cfg(feature = "sqlx-sqlite")]
mod real;
#[cfg(feature = "sqlx-sqlite")]
pub use real::*;

#[cfg(not(feature = "sqlx-sqlite"))]
mod mock;
#[cfg(not(feature = "sqlx-sqlite"))]
pub use mock::*;
