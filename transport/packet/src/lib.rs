pub mod errors;
pub mod v1;

pub mod prelude {
    pub use crate::v1::{ApplicationData, CustomTag, ReservedTag, ResolvedTag, Tag};
}
