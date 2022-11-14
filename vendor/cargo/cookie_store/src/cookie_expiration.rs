use std;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use time::{self, Tm};

#[derive(PartialEq, Eq, Clone, Debug, Hash, PartialOrd, Ord)]
pub struct SerializableTm(Tm);

impl Deref for SerializableTm {
    type Target = time::Tm;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Tm> for SerializableTm {
    fn from(tm: Tm) -> SerializableTm {
        SerializableTm(tm)
    }
}

/// When a given `Cookie` expires
#[derive(PartialEq, Eq, Clone, Debug, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CookieExpiration {
    /// `Cookie` expires at the given UTC time, as set from either the Max-Age
    /// or Expires attribute of a Set-Cookie header
    AtUtc(SerializableTm),
    /// `Cookie` expires at the end of the current `Session`; this means the cookie
    /// is not persistent
    SessionEnd,
}

impl CookieExpiration {
    /// Indicates if the `Cookie` is expired as of *now*.
    pub fn is_expired(&self) -> bool {
        self.expires_by(&time::now_utc())
    }

    /// Indicates if the `Cookie` expires as of `utc_tm`.
    pub fn expires_by(&self, utc_tm: &Tm) -> bool {
        match *self {
            CookieExpiration::AtUtc(ref expire_tm) => **expire_tm <= *utc_tm,
            CookieExpiration::SessionEnd => false,
        }
    }
}

impl From<u64> for CookieExpiration {
    fn from(max_age: u64) -> CookieExpiration {
        // If delta-seconds is less than or equal to zero (0), let expiry-time
        //    be the earliest representable date and time.  Otherwise, let the
        //    expiry-time be the current date and time plus delta-seconds seconds.
        let utc_tm = if 0 == max_age {
            time::at_utc(time::Timespec::new(0, 0))
        } else {
            // make sure we don't trigger a panic! in Duration by restricting the seconds
            // to the max
            let max_age = std::cmp::min(time::Duration::max_value().num_seconds() as u64, max_age);
            let utc_tm = time::now_utc() + time::Duration::seconds(max_age as i64);
            match time::strptime(&format!("{}", utc_tm.rfc3339()), "%Y-%m-%dT%H:%M:%SZ") {
                Ok(utc_tm) => utc_tm,
                Err(_) => time::strptime("9999-12-31T23:59:59Z", "%Y-%m-%dT%H:%M:%SZ")
                    .expect("unable to strptime maximum value"),
            }
        };
        CookieExpiration::from(utc_tm)
    }
}

impl From<time::Tm> for CookieExpiration {
    fn from(utc_tm: Tm) -> CookieExpiration {
        // format & re-parse the Tm to make sure de/serialization is consistent
        let utc_tm = match time::strptime(&format!("{}", utc_tm.rfc3339()), "%Y-%m-%dT%H:%M:%SZ") {
            Ok(utc_tm) => utc_tm,
            Err(_) => time::strptime("9999-12-31T23:59:59Z", "%Y-%m-%dT%H:%M:%SZ")
                .expect("unable to strptime maximum value"),
        };
        CookieExpiration::AtUtc(SerializableTm::from(utc_tm))
    }
}

impl From<time::Duration> for CookieExpiration {
    fn from(duration: time::Duration) -> Self {
        // If delta-seconds is less than or equal to zero (0), let expiry-time
        //    be the earliest representable date and time.  Otherwise, let the
        //    expiry-time be the current date and time plus delta-seconds seconds.
        let utc_tm = if duration.is_zero() {
            time::at_utc(time::Timespec::new(0, 0))
        } else {
            time::now_utc() + duration
        };
        CookieExpiration::from(utc_tm)
    }
}

#[cfg(test)]
mod tests {
    use super::CookieExpiration;
    use time;

    use crate::utils::test::*;

    #[test]
    fn max_age_bounds() {
        match CookieExpiration::from(time::Duration::max_value().num_seconds() as u64 + 1) {
            CookieExpiration::AtUtc(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn expired() {
        let ma = CookieExpiration::from(0u64); // Max-Age<=0 indicates the cookie is expired
        assert!(ma.is_expired());
        assert!(ma.expires_by(&in_days(-1)));
    }

    #[test]
    fn max_age() {
        let ma = CookieExpiration::from(60u64);
        assert!(!ma.is_expired());
        assert!(ma.expires_by(&in_minutes(2)));
    }

    #[test]
    fn session_end() {
        // SessionEnd never "expires"; lives until end of session
        let se = CookieExpiration::SessionEnd;
        assert!(!se.is_expired());
        assert!(!se.expires_by(&in_days(1)));
        assert!(!se.expires_by(&in_days(-1)));
    }

    #[test]
    fn at_utc() {
        {
            let expire_tmrw = CookieExpiration::from(in_days(1));
            assert!(!expire_tmrw.is_expired());
            assert!(expire_tmrw.expires_by(&in_days(2)));
        }
        {
            let expired_yest = CookieExpiration::from(in_days(-1));
            assert!(expired_yest.is_expired());
            assert!(!expired_yest.expires_by(&in_days(-2)));
        }
    }
}

mod serde_serialization {
    use super::SerializableTm;
    use serde;
    use serde::de::{Deserializer, Visitor};
    use std::fmt;
    use time;

    impl serde::Serialize for SerializableTm {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(&format!("{}", self.0.rfc3339()))
        }
    }

    impl<'a> serde::Deserialize<'a> for SerializableTm {
        fn deserialize<D>(deserializer: D) -> Result<SerializableTm, D::Error>
        where
            D: Deserializer<'a>,
        {
            deserializer.deserialize_str(TmVisitor)
        }
    }

    struct TmVisitor;

    impl<'a> Visitor<'a> for TmVisitor {
        type Value = SerializableTm;

        fn visit_str<E>(self, str_data: &str) -> Result<SerializableTm, E>
        where
            E: serde::de::Error,
        {
            time::strptime(str_data, "%Y-%m-%dT%H:%M:%SZ")
                .map(SerializableTm::from)
                .map_err(|_| {
                    E::custom(format!(
                        "could not parse '{}' as a UTC time in RFC3339 format",
                        str_data
                    ))
                })
        }

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("datetime")
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::cookie_expiration::CookieExpiration;
        use serde_json;
        use time;

        fn encode_decode(ce: &CookieExpiration, exp_json: &str) {
            let encoded = serde_json::to_string(ce).unwrap();
            assert!(
                exp_json == encoded,
                "expected: '{}'\n encoded: '{}'",
                exp_json,
                encoded
            );
            let decoded: CookieExpiration = serde_json::from_str(&encoded).unwrap();
            assert!(
                *ce == decoded,
                "expected: '{:?}'\n decoded: '{:?}'",
                ce,
                decoded
            );
        }

        #[test]
        fn serde() {
            let at_utc = time::strptime("2015-08-11T16:41:42Z", "%Y-%m-%dT%H:%M:%SZ").unwrap();
            encode_decode(
                &CookieExpiration::from(at_utc),
                "{\"AtUtc\":\"2015-08-11T16:41:42Z\"}",
            );
            encode_decode(&CookieExpiration::SessionEnd, "\"SessionEnd\"");
        }
    }
}
