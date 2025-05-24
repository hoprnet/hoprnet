pub mod errors;
pub mod v1;

pub mod prelude {
    use std::ops::Range;

    pub use crate::v1::{ApplicationData, CustomTag, ReservedTag, ResolvedTag, Tag};

    pub trait TagRangeExt {
        const USABLE_RANGE: Range<Tag>;
    }
}
