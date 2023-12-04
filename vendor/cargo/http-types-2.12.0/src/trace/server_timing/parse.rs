use std::time::Duration;

use super::Metric;
use crate::{ensure, format_err, StatusCode};

/// Parse multiple entries from a single header.
///
/// Each entry is comma-delimited.
pub(super) fn parse_header(s: &str, entries: &mut Vec<Metric>) -> crate::Result<()> {
    for part in s.trim().split(',') {
        let entry = parse_entry(part).map_err(|mut e| {
            e.set_status(StatusCode::BadRequest);
            e
        })?;
        entries.push(entry);
    }
    Ok(())
}

/// Create an entry from a string. Parsing rules in ABNF are:
//
/// ```txt
/// Server-Timing             = #server-timing-metric
/// server-timing-metric      = metric-name *( OWS ";" OWS server-timing-param )
/// metric-name               = token
/// server-timing-param       = server-timing-param-name OWS "=" OWS server-timing-param-value
/// server-timing-param-name  = token
/// server-timing-param-value = token / quoted-string
/// ```
//
/// Source: https://w3c.github.io/server-timing/#the-server-timing-header-field
fn parse_entry(s: &str) -> crate::Result<Metric> {
    let mut parts = s.trim().split(';');

    // Get the name. This is non-optional.
    let name = parts
        .next()
        .ok_or_else(|| format_err!("Server timing headers must include a name"))?
        .trim_end();

    // We must extract these values from the k-v pairs that follow.
    let mut dur = None;
    let mut desc = None;

    for mut part in parts {
        ensure!(
            !part.is_empty(),
            "Server timing params cannot end with a trailing `;`"
        );

        part = part.trim_start();

        let mut params = part.split('=');
        let name = params
            .next()
            .ok_or_else(|| format_err!("Server timing params must have a name"))?
            .trim_end();
        let mut value = params
            .next()
            .ok_or_else(|| format_err!("Server timing params must have a value"))?
            .trim_start();

        match name {
            "dur" => {
                let millis: f64 = value.parse().map_err(|_| {
                        format_err!("Server timing duration params must be a valid double-precision floating-point number.")
                    })?;
                dur = Some(Duration::from_secs_f64(millis / 1000.0));
            }
            "desc" => {
                // Ensure quotes line up, and strip them from the resulting output
                if value.starts_with('"') {
                    value = &value[1..value.len()];
                    ensure!(
                        value.ends_with('"'),
                        "Server timing description params must use matching quotes"
                    );
                    value = &value[0..value.len() - 1];
                } else {
                    ensure!(
                        !value.ends_with('"'),
                        "Server timing description params must use matching quotes"
                    );
                }
                desc = Some(value.to_string());
            }
            _ => continue,
        }
    }

    Ok(Metric {
        name: name.to_string(),
        dur,
        desc,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decode_header() -> crate::Result<()> {
        // Metric name only.
        assert_entry("Server", "Server", None, None)?;
        assert_entry("Server ", "Server", None, None)?;
        assert_entry_err(
            "Server ;",
            "Server timing params cannot end with a trailing `;`",
        );
        assert_entry_err(
            "Server; ",
            "Server timing params cannot end with a trailing `;`",
        );

        // Metric name + param
        assert_entry("Server; dur=1000", "Server", Some(1000), None)?;
        assert_entry("Server; dur =1000", "Server", Some(1000), None)?;
        assert_entry("Server; dur= 1000", "Server", Some(1000), None)?;
        assert_entry("Server; dur = 1000", "Server", Some(1000), None)?;
        assert_entry_err(
            "Server; dur=1000;",
            "Server timing params cannot end with a trailing `;`",
        );

        // Metric name + desc
        assert_entry(r#"DB; desc="a db""#, "DB", None, Some("a db"))?;
        assert_entry(r#"DB; desc ="a db""#, "DB", None, Some("a db"))?;
        assert_entry(r#"DB; desc= "a db""#, "DB", None, Some("a db"))?;
        assert_entry(r#"DB; desc = "a db""#, "DB", None, Some("a db"))?;
        assert_entry(r#"DB; desc=a_db"#, "DB", None, Some("a_db"))?;
        assert_entry_err(
            r#"DB; desc="db"#,
            "Server timing description params must use matching quotes",
        );
        assert_entry_err(
            "Server; desc=a_db;",
            "Server timing params cannot end with a trailing `;`",
        );

        // Metric name + dur + desc
        assert_entry(
            r#"Server; dur=1000; desc="a server""#,
            "Server",
            Some(1000),
            Some("a server"),
        )?;
        assert_entry_err(
            r#"Server; dur=1000; desc="a server";"#,
            "Server timing params cannot end with a trailing `;`",
        );
        Ok(())
    }

    #[test]
    fn decode_headers() -> crate::Result<()> {
        // Example from MDN.
        let mut entries = vec![];
        parse_header("db;dur=53, app;dur=47.2", &mut entries)?;
        let e = &entries[0];
        assert_eq!(e.name(), "db");
        assert_eq!(e.duration(), Some(Duration::from_millis(53)));
        let e = &entries[1];
        assert_eq!(e.name(), "app");
        assert_eq!(e.duration(), Some(Duration::from_micros(47200)));
        Ok(())
    }

    fn assert_entry_err(s: &str, msg: &str) {
        let err = parse_entry(s).unwrap_err();
        assert_eq!(format!("{}", err), msg);
    }

    /// Assert an entry and all of its fields.
    fn assert_entry(s: &str, n: &str, du: Option<u64>, de: Option<&str>) -> crate::Result<()> {
        let e = parse_entry(s)?;
        assert_eq!(e.name(), n);
        assert_eq!(e.duration(), du.map(Duration::from_millis));
        assert_eq!(e.description(), de);
        Ok(())
    }
}
