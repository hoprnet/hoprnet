//! Helper functions for multipart/form-data support

use hyper::header::{HeaderMap, CONTENT_TYPE};

/// Utility function to get the multipart boundary marker (if any) from the Headers.
pub fn boundary(headers: &HeaderMap) -> Option<String> {
    headers.get(CONTENT_TYPE).and_then(|content_type| {
        match content_type.to_str() {
            Ok(val) => val.parse::<mime::Mime>().ok(),
            _ => None,
        }
        .and_then(|ref mime| {
            if mime.type_() == mime::MULTIPART && mime.subtype() == mime::FORM_DATA {
                mime.get_param(mime::BOUNDARY).map(|x| x.to_string())
            } else {
                None
            }
        })
    })
}
