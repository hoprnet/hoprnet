use crate::ensure;
use crate::headers::HeaderValue;
use crate::Mime;

use std::ops::{Deref, DerefMut};
use std::{
    cmp::{Ordering, PartialEq},
    str::FromStr,
};

/// A proposed Media Type for the `Accept` header.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaTypeProposal {
    /// The proposed media_type.
    pub(crate) media_type: Mime,

    /// The weight of the proposal.
    ///
    /// This is a number between 0.0 and 1.0, and is max 3 decimal points.
    weight: Option<f32>,
}

impl MediaTypeProposal {
    /// Create a new instance of `MediaTypeProposal`.
    pub fn new(media_type: impl Into<Mime>, weight: Option<f32>) -> crate::Result<Self> {
        if let Some(weight) = weight {
            ensure!(
                weight.is_sign_positive() && weight <= 1.0,
                "MediaTypeProposal should have a weight between 0.0 and 1.0"
            )
        }

        Ok(Self {
            media_type: media_type.into(),
            weight,
        })
    }

    /// Get the proposed media_type.
    pub fn media_type(&self) -> &Mime {
        &self.media_type
    }

    /// Get the weight of the proposal.
    pub fn weight(&self) -> Option<f32> {
        self.weight
    }

    /// Parse a string into a media type proposal.
    ///
    /// Because `;` and `q=0.0` are all valid values for in use in a media type,
    /// we have to parse the full string to the media type first, and then see if
    /// a `q` value has been set.
    pub(crate) fn from_str(s: &str) -> crate::Result<Self> {
        let mut media_type = Mime::from_str(s)?;
        let weight = media_type
            .remove_param("q")
            .map(|param| param.as_str().parse())
            .transpose()?;
        Self::new(media_type, weight)
    }
}

impl From<Mime> for MediaTypeProposal {
    fn from(media_type: Mime) -> Self {
        Self {
            media_type,
            weight: None,
        }
    }
}

impl From<MediaTypeProposal> for Mime {
    fn from(accept: MediaTypeProposal) -> Self {
        accept.media_type
    }
}

impl PartialEq<Mime> for MediaTypeProposal {
    fn eq(&self, other: &Mime) -> bool {
        self.media_type == *other
    }
}

impl PartialEq<Mime> for &MediaTypeProposal {
    fn eq(&self, other: &Mime) -> bool {
        self.media_type == *other
    }
}

impl Deref for MediaTypeProposal {
    type Target = Mime;
    fn deref(&self) -> &Self::Target {
        &self.media_type
    }
}

impl DerefMut for MediaTypeProposal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.media_type
    }
}

// NOTE: For Accept-Encoding Firefox sends the values: `gzip, deflate, br`. This means
// when parsing media_types we should choose the last value in the list under
// equal weights. This impl doesn't know which value was passed later, so that
// behavior needs to be handled separately.
//
// NOTE: This comparison does not include a notion of `*` (any value is valid).
// that needs to be handled separately.
impl PartialOrd for MediaTypeProposal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self.weight, other.weight) {
            (Some(left), Some(right)) => left.partial_cmp(&right),
            (Some(_), None) => Some(Ordering::Greater),
            (None, Some(_)) => Some(Ordering::Less),
            (None, None) => None,
        }
    }
}

impl From<MediaTypeProposal> for HeaderValue {
    fn from(entry: MediaTypeProposal) -> HeaderValue {
        let s = match entry.weight {
            Some(weight) => format!("{};q={:.3}", entry.media_type, weight),
            None => entry.media_type.to_string(),
        };
        unsafe { HeaderValue::from_bytes_unchecked(s.into_bytes()) }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mime;

    #[test]
    fn smoke() {
        let _ = MediaTypeProposal::new(mime::JSON, Some(0.0)).unwrap();
        let _ = MediaTypeProposal::new(mime::XML, Some(0.5)).unwrap();
        let _ = MediaTypeProposal::new(mime::HTML, Some(1.0)).unwrap();
    }

    #[test]
    fn error_code_500() {
        let err = MediaTypeProposal::new(mime::JSON, Some(1.1)).unwrap_err();
        assert_eq!(err.status(), 500);

        let err = MediaTypeProposal::new(mime::XML, Some(-0.1)).unwrap_err();
        assert_eq!(err.status(), 500);

        let err = MediaTypeProposal::new(mime::HTML, Some(-0.0)).unwrap_err();
        assert_eq!(err.status(), 500);
    }
}
