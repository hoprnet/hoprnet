pub mod protocol;
pub mod state;
pub mod errors;

#[cfg(test)]
mod tests {
    #[test]
    fn test_bitmap() {
        const UBOUND: u16 = 3592;
        // Max 3592 segments of 456 bytes each (44 bytes header total from 500 MTU).
        // This yields max 1599 kB frame size.
    }
}
