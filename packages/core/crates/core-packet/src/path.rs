use libp2p_identity::PeerId;

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct Path {
    hops: Vec<PeerId>,
    valid: bool,
}

impl Path {
    pub fn new_valid(validated_path: Vec<PeerId>) -> Self {
        Self {
            hops: validated_path,
            valid: true,
        }
    }

    pub fn length(&self) -> usize {
        self.hops.len()
    }

    pub fn hops(&self) -> Vec<&PeerId> {
        self.hops.iter().collect::<Vec<_>>()
    }

    pub fn valid(&self) -> bool {
        self.valid
    }
}

#[cfg(test)]
mod tests {
    use crate::path::Path;
    use libp2p_identity::PeerId;

    #[test]
    fn test_path_validated() {
        const HOPS: usize = 5;
        let peer_ids = (0..HOPS).map(|_| PeerId::random()).collect::<Vec<_>>();

        let path = Path::new_valid(peer_ids.clone());
        assert_eq!(HOPS, path.length());
        assert_eq!(peer_ids.iter().collect::<Vec<_>>(), path.hops());
        assert!(path.valid());
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::errors::PacketError::Other;
    use crate::errors::Result;
    use crate::path::Path;
    use js_sys::JsString;
    use libp2p_identity::PeerId;
    use std::str::FromStr;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::errors::GeneralError::ParseError;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    impl Path {
        #[wasm_bindgen(constructor)]
        pub fn _new_validated(validated_path: Vec<JsString>) -> JsResult<Path> {
            Ok(Path::new_valid(ok_or_jserr!(validated_path
                .into_iter()
                .map(|p| PeerId::from_str(&p.as_string().unwrap()).map_err(|_| Other(ParseError)))
                .collect::<Result<Vec<PeerId>>>())?))
        }
    }
}
