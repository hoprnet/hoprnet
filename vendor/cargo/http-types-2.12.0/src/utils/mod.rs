mod date;

pub(crate) use date::fmt_http_date;
pub(crate) use date::parse_http_date;
pub(crate) use date::HttpDate;

use crate::{Error, Status, StatusCode};

use std::cmp::Ordering;
use std::str::FromStr;

/// Parse a weight of the form `q=0.123`.
pub(crate) fn parse_weight(s: &str) -> crate::Result<f32> {
    let mut parts = s.split('=');
    if !matches!(parts.next(), Some("q")) {
        let mut err = Error::new_adhoc("invalid weight");
        err.set_status(StatusCode::BadRequest);
        return Err(err);
    }
    match parts.next() {
        Some(s) => {
            let weight = f32::from_str(s).status(400)?;
            Ok(weight)
        }
        None => {
            let mut err = Error::new_adhoc("invalid weight");
            err.set_status(StatusCode::BadRequest);
            Err(err)
        }
    }
}

/// Order proposals by weight. Try ordering by q value first. If equal or undefined,
/// order by index, favoring the latest provided value.
pub(crate) fn sort_by_weight<T: PartialOrd + Clone>(props: &mut Vec<T>) {
    let mut arr: Vec<(usize, T)> = props.iter().cloned().enumerate().collect();
    arr.sort_unstable_by(|a, b| match b.1.partial_cmp(&a.1) {
        None | Some(Ordering::Equal) => b.0.cmp(&a.0),
        Some(ord) => ord,
    });
    *props = arr.into_iter().map(|(_, t)| t).collect::<Vec<T>>();
}
