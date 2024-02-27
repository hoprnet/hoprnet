#[cfg(feature = "sqlx-mysql")]
mod real;
#[cfg(feature = "sqlx-mysql")]
pub use real::*;

#[cfg(not(feature = "sqlx-mysql"))]
mod mock;
#[cfg(not(feature = "sqlx-mysql"))]
pub use mock::*;
