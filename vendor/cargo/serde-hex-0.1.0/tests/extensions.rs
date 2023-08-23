//! Test of extension traits (e.g.; `SerHexSeq`).
extern crate serde_hex;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use serde_hex::{SerHexOpt, SerHexSeq, StrictPfx};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Ext {
    #[serde(with = "SerHexSeq::<StrictPfx>")]
    seq: Vec<u8>,
    #[serde(with = "SerHexOpt::<StrictPfx>")]
    opt: Option<u8>,
}

#[test]
fn serialize() {
    let ext = Ext {
        seq: vec![0xde, 0xad, 0xbe, 0xef],
        opt: Some(0xff),
    };
    let ser = serde_json::to_string(&ext).unwrap();
    let exp = r#"{"seq":"0xdeadbeef","opt":"0xff"}"#;
    assert_eq!(ser, exp);
}

#[test]
fn deserialize() {
    let ser = r#"{"seq":"0x0123456789abcdef","opt":"aa"}"#;
    let ext = serde_json::from_str::<Ext>(ser).unwrap();
    let exp = Ext {
        seq: vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
        opt: Some(0xaa),
    };
    assert_eq!(ext, exp);
}
