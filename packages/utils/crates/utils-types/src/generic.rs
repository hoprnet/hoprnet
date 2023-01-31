pub trait TypeBase: PartialEq + ToString {

    fn to_hex(&self) -> String;
}