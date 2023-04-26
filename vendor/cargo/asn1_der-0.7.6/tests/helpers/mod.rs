use asn1_der::{
    Asn1DerError,
    Asn1DerErrorVariant::{InOutError, InvalidData, Other, Unsupported},
};

pub trait OptionExt<T> {
    /// Returns the `Some` variant or pretty prints the error and panics
    fn assert(self, name: &str) -> T;
    /// Returns the `Some` variant or pretty prints the error and panics
    fn assert_index(self, name: &str, i: usize) -> T;
}
impl<T> OptionExt<T> for Option<T> {
    fn assert(self, name: &str) -> T {
        self.unwrap_or_else(|| {
            eprintln!("Unexpectet `None` result @\"{}\"", name);
            panic!("Panicked due to fatal error");
        })
    }
    fn assert_index(self, name: &str, i: usize) -> T {
        self.unwrap_or_else(|| {
            eprintln!("Unexpected `None` result @\"{}\":{}", name, i);
            panic!("Panicked due to fatal error");
        })
    }
}

pub trait ResultExt<T, E> {
    /// Returns the `Ok` variant or pretty prints the error and panics
    fn assert(self, name: &str) -> T;
    /// Returns the `Ok` variant or pretty prints the error and panics
    fn assert_index(self, name: &str, i: usize) -> T;
    /// Ensures that the result is an `Err` of type `variant`
    fn assert_err(self, variant: &str, name: &str);
}
impl<T> ResultExt<T, Asn1DerError> for Result<T, Asn1DerError> {
    fn assert(self, name: &str) -> T {
        self.unwrap_or_else(|e| {
            eprintln!("Fatal error @\"{}\": {}", name, e);
            panic!("Panicked due to fatal error");
        })
    }
    fn assert_index(self, name: &str, i: usize) -> T {
        self.unwrap_or_else(|e| {
            eprintln!("Fatal error @\"{}\":{}: {}", name, i, e);
            panic!("Panicked due to fatal error");
        })
    }
    fn assert_err(self, variant: &str, name: &str) {
        match self {
            Err(Asn1DerError { error: InOutError(_), .. }) if variant == "InOutError" => (),
            Err(Asn1DerError { error: InvalidData(_), .. }) if variant == "InvalidData" => (),
            Err(Asn1DerError { error: Unsupported(_), .. }) if variant == "Unsupported" => (),
            Err(Asn1DerError { error: Other(_), .. }) if variant == "Other" => (),
            Ok(_) => {
                eprintln!("Unexpected success @\"{}\"; expected {}", name, variant);
                panic!("Panicked due to unexpected success")
            }
            Err(e) => {
                eprintln!("Unexpected error kind @\"{}\"; got {}\ninstead of {}", name, e, variant);
                panic!("Panicked due to invalid error kind");
            }
        }
    }
}

pub mod test_ok {
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Length {
        pub name: String,
        pub bytes: Vec<u8>,
        pub value: Option<u64>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Object {
        pub name: String,
        pub bytes: Vec<u8>,
        pub tag: u8,
        pub value: Vec<u8>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct TypedBool {
        pub name: String,
        pub bytes: Vec<u8>,
        pub value: Vec<u8>,
        pub bool: bool,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct TypedInteger {
        pub name: String,
        pub bytes: Vec<u8>,
        pub value: Vec<u8>,
        pub uint: Option<u128>,
        pub int: Option<i128>,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct TypedNull {
        pub name: String,
        pub bytes: Vec<u8>,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct TypedOctetString {
        pub name: String,
        pub bytes: Vec<u8>,
        pub value: Vec<u8>,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct TypedSequence {
        pub name: String,
        pub bytes: Vec<u8>,
        pub value: Vec<u8>,
        pub sequence: Vec<Object>,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct TypedUtf8String {
        pub name: String,
        pub bytes: Vec<u8>,
        pub value: Vec<u8>,
        pub utf8str: String,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Typed {
        pub bool: Vec<TypedBool>,
        pub integer: Vec<TypedInteger>,
        pub null: Vec<TypedNull>,
        pub octet_string: Vec<TypedOctetString>,
        pub sequence: Vec<TypedSequence>,
        pub utf8_string: Vec<TypedUtf8String>,
    }

    /// A test vector for valid constructions
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Test {
        pub length: Vec<Length>,
        pub object: Vec<Object>,
        pub typed: Typed,
    }
    /// Loads the test vectors for valid constructions
    pub fn load() -> Test {
        serde_json::from_str(include_str!("../ok.json")).expect("Failed to load test vectors")
    }
}

pub mod test_err {
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Length {
        pub name: String,
        pub bytes: Vec<u8>,
        pub err: String,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Object {
        pub name: String,
        pub bytes: Vec<u8>,
        err: String,
        err_32bit: Option<String>,
    }
    impl Object {
        /// Gets the platform dependent error
        pub fn err(&self) -> &str {
            match self.err_32bit.as_ref() {
                #[cfg(target_pointer_width = "32")]
                Some(err_32bit) => err_32bit,
                _ => &self.err,
            }
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct TypedAny {
        pub name: String,
        pub bytes: Vec<u8>,
        pub err: String,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Typed {
        pub bool: Vec<TypedAny>,
        pub integer: Vec<TypedAny>,
        pub null: Vec<TypedAny>,
        pub octet_string: Vec<TypedAny>,
        pub sequence: Vec<TypedAny>,
        pub utf8_string: Vec<TypedAny>,
    }

    /// A test vector for invalid constructions
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Test {
        pub length: Vec<Length>,
        pub object: Vec<Object>,
        pub typed: Typed,
    }
    /// Loads the test vectors for invalid constructions
    pub fn load() -> Test {
        serde_json::from_str(include_str!("../err.json")).expect("Failed to load test vectors")
    }
}
