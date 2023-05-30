use libp2p_identity::PeerId;
use std::fmt::{Display, Formatter};

/// Represents a path for a packet.
/// The type internally carries an information if the path has been already validated or not (since path validation
/// is potentially an expensive operation).
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct Path {
    hops: Vec<PeerId>,
    valid: bool,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Path {
    /// Number of hops in the path.
    pub fn length(&self) -> u32 {
        self.hops.len() as u32
    }

    /// Determines with the path is valid.
    pub fn valid(&self) -> bool {
        self.valid
    }
}

impl Path {
    /// Creates an already pre-validated path.
    pub fn new_valid(validated_path: Vec<PeerId>) -> Self {
        Self {
            hops: validated_path,
            valid: true,
        }
    }

    /// Individual hops in the path.
    pub fn hops(&self) -> Vec<&PeerId> {
        self.hops.iter().collect::<Vec<_>>()
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}", if self.valid { " " } else { " !! " })?;
        for peer in &self.hops {
            write!(f, "{peer}->")?;
        }
        write!(f, " ]")
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
