pub trait Name {
    fn from_str(string: &str) -> Option<Self>
    where
        Self: Sized;
}
