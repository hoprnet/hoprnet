#[derive(Clone, Debug)]
pub struct Color {}

impl Color {
    #[inline]
    pub const fn truecolor(&self) -> bool {
        false
    }

    #[inline]
    pub const fn color(&self) -> bool {
        false
    }

    #[inline]
    pub const fn ansi_color(&self) -> bool {
        false
    }
}

pub fn get(_stream: crate::Stream) -> Color {
    Color {}
}

#[cfg(feature = "api_unstable")]
pub fn set(_choice: crate::ColorChoice) {}
