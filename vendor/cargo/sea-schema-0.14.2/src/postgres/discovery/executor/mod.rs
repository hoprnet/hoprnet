#[cfg(feature = "sqlx-postgres")]
mod real;
#[cfg(feature = "sqlx-postgres")]
pub use real::*;

#[cfg(not(feature = "sqlx-postgres"))]
mod mock;
#[cfg(not(feature = "sqlx-postgres"))]
pub use mock::*;
