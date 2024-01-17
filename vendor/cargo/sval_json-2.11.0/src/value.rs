use core::fmt;

/**
A string containing encoded JSON.

Streaming a `JsonStr` will embed its contents directly rather
than treating them as a string.
*/
#[repr(transparent)]
#[derive(PartialEq, Eq, Hash)]
pub struct JsonStr(str);

impl JsonStr {
    /**
    Treat a string as native JSON.
    */
    pub const fn new<'a>(json: &'a str) -> &'a Self {
        // SAFETY: `JsonStr` and `str` have the same ABI
        unsafe { &*(json as *const _ as *const JsonStr) }
    }

    /**
    Get a reference to the underlying string.
    */
    pub const fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for JsonStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for JsonStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl PartialEq<str> for JsonStr {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl sval::Value for JsonStr {
    fn stream<'sval, S: sval::Stream<'sval> + ?Sized>(&'sval self, stream: &mut S) -> sval::Result {
        stream.tagged_begin(Some(&crate::tags::JSON_VALUE), None, None)?;
        stream.value(&self.0)?;
        stream.tagged_end(Some(&crate::tags::JSON_VALUE), None, None)
    }
}

#[cfg(feature = "alloc")]
mod alloc_support {
    use super::*;

    use alloc::boxed::Box;

    impl JsonStr {
        /**
        Treat a string as native JSON.
        */
        pub fn boxed(json: impl Into<Box<str>>) -> Box<Self> {
            let json = json.into();

            // SAFETY: `JsonStr` and `str` have the same ABI
            unsafe { Box::from_raw(Box::into_raw(json) as *mut str as *mut JsonStr) }
        }
    }
}
