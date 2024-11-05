#[cfg(feature = "sqlx-dep")]
mod real;
#[cfg(feature = "sqlx-dep")]
pub use real::*;

#[cfg(not(feature = "sqlx-dep"))]
mod mock;
#[cfg(not(feature = "sqlx-dep"))]
pub use mock::*;
