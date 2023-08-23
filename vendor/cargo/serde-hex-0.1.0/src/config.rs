//! `SerHex` configuration variants.
//!
//! This module is a collection of marker types which implement
//! the possible combinations of config values described by the
//! `HexConf` trait.  All config values are supplied with associated
//! functions that are marked with `#[inline]`.  This ensures that
//! the compiler will (usually) optimize away all configuration
//! checks.

/// Trait for supplying configuration to `SerHex`.
/// This trait takes no `self` parameters, as it is
/// intended to be applied unit structs.  All default
/// implementation are set to `false`.
pub trait HexConf {
    /// function indicating whether to use compact
    /// (as apposed to strict) representation.
    #[inline]
    fn compact() -> bool {
        false
    }
    /// function indicating whether to prefixing (`0x`).
    #[inline]
    fn withpfx() -> bool {
        false
    }
    /// function indicating whether to use capital letters (`A-F`).
    #[inline]
    fn withcap() -> bool {
        false
    }
}

/// Config indicating a strict representation
/// with no capiltaization and no prefixing.
pub struct Strict;
impl HexConf for Strict {}

/// Config indicating a strict representation
/// with prefixing but no capitalization.
pub struct StrictPfx;
impl HexConf for StrictPfx {
    #[inline]
    fn withpfx() -> bool {
        true
    }
}

/// Config indicating a strict representation
/// with capitalization but no prefixing.
pub struct StrictCap;
impl HexConf for StrictCap {
    #[inline]
    fn withcap() -> bool {
        true
    }
}

/// Config indicating a strict representation
/// with capitalization and prefixing.
pub struct StrictCapPfx;
impl HexConf for StrictCapPfx {
    #[inline]
    fn withpfx() -> bool {
        true
    }
    #[inline]
    fn withcap() -> bool {
        true
    }
}

/// Config indicating compact representation
/// with no capitalization and no prefixing.
pub struct Compact;
impl HexConf for Compact {
    #[inline]
    fn compact() -> bool {
        true
    }
}

/// Config indicating compact representation
/// with prefixing but no capitalization.
pub struct CompactPfx;
impl HexConf for CompactPfx {
    #[inline]
    fn compact() -> bool {
        true
    }
    #[inline]
    fn withpfx() -> bool {
        true
    }
}

/// Config indicating compact representation
/// with capitalization but no prefixing.
pub struct CompactCap;
impl HexConf for CompactCap {
    #[inline]
    fn compact() -> bool {
        true
    }
    #[inline]
    fn withcap() -> bool {
        true
    }
}

/// Config indicating compact representation
/// with capitalization and prefixing.
pub struct CompactCapPfx;
impl HexConf for CompactCapPfx {
    #[inline]
    fn compact() -> bool {
        true
    }
    #[inline]
    fn withcap() -> bool {
        true
    }
    #[inline]
    fn withpfx() -> bool {
        true
    }
}
