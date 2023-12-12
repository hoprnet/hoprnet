//! Helper functions for multipart/related support

use hyper_0_10::header::{ContentType, Headers};
use mime_0_2::Mime;

/// Construct the Body for a multipart/related request. The mime 0.2.6 library
/// does not parse quoted-string parameters correctly. The boundary doesn't
/// need to be a quoted string if it does not contain a '/', hence ensure
/// no such boundary is used.
pub fn generate_boundary() -> Vec<u8> {
    let mut boundary = mime_multipart::generate_boundary();
    for b in boundary.iter_mut() {
        if *b == b'/' {
            *b = b'.';
        }
    }

    boundary
}

/// Create the multipart headers from a request so that we can parse the
/// body using `mime_multipart::read_multipart_body`.
pub fn create_multipart_headers(
    content_type: Option<&hyper::header::HeaderValue>,
) -> Result<Headers, String> {
    let content_type = content_type
        .ok_or_else(|| "Missing Content-Type header".to_string())?
        .to_str()
        .map_err(|e| format!("Couldn't read Content-Type header value: {}", e))?
        .parse::<Mime>()
        .map_err(|_e| "Couldn't parse Content-Type header value".to_string())?;

    // Insert top-level content type header into a Headers object.
    let mut multipart_headers = Headers::new();
    multipart_headers.set(ContentType(content_type));

    Ok(multipart_headers)
}
