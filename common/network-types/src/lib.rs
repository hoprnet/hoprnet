/// Contains all errors thrown from this crate.
pub mod errors;
/// Types related to packet segmentation and reassembly
pub mod frame;

#[doc(hidden)]
pub mod prelude {
    pub use crate::frame::{Frame, FrameReassembler, Segment};
}
