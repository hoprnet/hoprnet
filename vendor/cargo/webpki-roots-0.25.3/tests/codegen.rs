use std::ascii::escape_default;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;
use std::fs;

use chrono::{NaiveDate, Utc};
use num_bigint::BigUint;
use ring::digest;
use serde::Deserialize;
use x509_parser::prelude::AttributeTypeAndValue;
use x509_parser::x509::X509Name;

#[tokio::test]
async fn new_generated_code_is_fresh() {
    // Configure a Reqwest client that only trusts the CA certificate expected to be the
    // root of trust for the CCADB server.
    //
    // If we see Unknown CA TLS validation failures from the Reqwest client in the future it
    // likely indicates that the upstream service has changed certificate authorities. In this
    // case the vendored root CA will need to be updated. You can find the current root in use with
    // Chrome by:
    //  1. Navigating to `https://ccadb-public.secure.force.com/mozilla/`
    //  2. Clicking the lock icon.
    //  3. Clicking "Connection is secure"
    //  4. Clicking "Certificate is valid"
    //  5. Clicking the "Details" tab.
    //  6. Selecting the topmost "System Trust" entry.
    //  7. Clicking "Export..." and saving the certificate to `webpki-roots/tests/data/`.
    //  8. Committing the updated .pem root CA, and updating the `include_bytes!` path.
    let root = include_bytes!("data/DigiCertGlobalRootCA.pem");
    let root = reqwest::Certificate::from_pem(root).unwrap();
    let client = reqwest::Client::builder()
        .user_agent(format!("webpki-roots/v{}", env!("CARGO_PKG_VERSION")))
        .add_root_certificate(root)
        .build()
        .unwrap();

    let ccadb_url =
        "https://ccadb-public.secure.force.com/mozilla/IncludedCACertificateReportPEMCSV";
    eprintln!("fetching {ccadb_url}...");

    let req = client.get(ccadb_url).build().unwrap();
    let csv_data = client
        .execute(req)
        .await
        .expect("failed to fetch CSV")
        .text()
        .await
        .unwrap();

    // Parse the CSV metadata.
    let metadata = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_data.as_bytes())
        .into_deserialize::<CertificateMetadata>()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    // Filter for just roots with the TLS trust bit that are not distrusted as of today's date.
    let trusted_tls_roots = metadata
        .into_iter()
        .filter(|root| root.trusted_for_tls(&Utc::now().naive_utc().date()))
        .collect::<Vec<CertificateMetadata>>();

    // Create an ordered BTreeMap of the roots, panicking for any duplicates.
    let mut tls_roots_map = BTreeMap::new();
    for root in trusted_tls_roots {
        match tls_roots_map.get(&root.sha256_fingerprint) {
            Some(_) => {
                panic!("duplicate fingerprint {}", root.sha256_fingerprint);
            }
            None => {
                tls_roots_map.insert(root.sha256_fingerprint.clone(), root);
            }
        }
    }

    let mut code = String::with_capacity(256 * 1_024);
    code.push_str(HEADER);
    code.push_str("pub const TLS_SERVER_ROOTS: &[TrustAnchor] = &[\n");
    let (mut subject, mut spki, mut name_constraints) =
        (String::new(), String::new(), String::new());

    for (_, root) in tls_roots_map {
        // Verify the DER FP matches the metadata FP.
        let der = root.der();
        let calculated_fp = digest::digest(&digest::SHA256, &der);
        let metadata_fp = hex::decode(&root.sha256_fingerprint).expect("malformed fingerprint");
        assert_eq!(calculated_fp.as_ref(), metadata_fp.as_slice());

        let ta = webpki::TrustAnchor::try_from_cert_der(&der).expect("malformed trust anchor der");
        subject.clear();
        for &b in ta.subject {
            write!(&mut subject, "{}", escape_default(b)).unwrap();
        }

        spki.clear();
        for &b in ta.spki {
            write!(&mut spki, "{}", escape_default(b)).unwrap();
        }

        name_constraints.clear();
        if let Some(nc) = &root.mozilla_applied_constraints() {
            for &b in nc.iter() {
                write!(&mut name_constraints, "{}", escape_default(b)).unwrap();
            }
        }

        let (_, parsed_cert) =
            x509_parser::parse_x509_certificate(&der).expect("malformed x509 der");
        let issuer = name_to_string(parsed_cert.issuer());
        let subject_str = name_to_string(parsed_cert.subject());
        let label = root.common_name_or_certificate_name.clone();
        let serial = root.serial().to_string();
        let sha256_fp = root.sha256_fp();

        // Write comment
        code.push_str("  /*\n");
        code.push_str(&format!("   * Issuer: {}\n", issuer));
        code.push_str(&format!("   * Subject: {}\n", subject_str));
        code.push_str(&format!("   * Label: {:?}\n", label));
        code.push_str(&format!("   * Serial: {}\n", serial));
        code.push_str(&format!("   * SHA256 Fingerprint: {}\n", sha256_fp));
        for ln in root.pem().lines() {
            code.push_str("   * ");
            code.push_str(ln.trim());
            code.push('\n');
        }
        code.push_str("   */\n");

        // Write the code
        code.push_str("  TrustAnchor {\n");
        code.write_fmt(format_args!("    subject: b\"{subject}\",\n"))
            .unwrap();
        code.write_fmt(format_args!("    spki: b\"{spki}\",\n"))
            .unwrap();
        match name_constraints.is_empty() {
            false => code
                .write_fmt(format_args!(
                    "    name_constraints: Some(b\"{name_constraints}\")\n"
                ))
                .unwrap(),
            true => code.push_str("    name_constraints: None\n"),
        }
        code.push_str("  },\n\n");
    }
    code.push_str("];\n");

    // Check that the generated code matches the checked-in code
    let old = fs::read_to_string("src/lib.rs").unwrap();
    if old != code {
        fs::write("src/lib.rs", code).unwrap();
        panic!("generated code changed");
    }
}

/// The built-in x509_parser::X509Name Display impl uses a different sort order than
/// the one historically used by mkcert.org^[0]. We re-create that sort order here to
/// avoid unnecessary churn in the generated code.
///
/// [0]: <https://github.com/Lukasa/mkcert/blob/6911a8f68681f4d6a795c1f6db7b063f75b03b5a/certs/convert_mozilla_certdata.go#L405-L428>
fn name_to_string(name: &X509Name) -> String {
    let mut ret = String::with_capacity(256);

    if let Some(cn) = name
        .iter_common_name()
        .next()
        .and_then(|cn| cn.as_str().ok())
    {
        write!(ret, "CN={}", cn).unwrap();
    }

    let mut append_attrs = |attrs: Vec<&AttributeTypeAndValue>, label| {
        let str_parts = attrs
            .iter()
            .filter_map(|attr| match attr.as_str() {
                Ok(s) => Some(s),
                Err(_) => None,
            })
            .collect::<Vec<_>>()
            .join("/");
        if !str_parts.is_empty() {
            if !ret.is_empty() {
                ret.push(' ');
            }
            write!(ret, "{}={}", label, str_parts).unwrap();
        }
    };

    append_attrs(name.iter_organization().collect(), "O");
    append_attrs(name.iter_organizational_unit().collect(), "OU");

    ret
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Deserialize)]
pub struct CertificateMetadata {
    #[serde(rename = "Common Name or Certificate Name")]
    pub common_name_or_certificate_name: String,

    #[serde(rename = "Certificate Serial Number")]
    pub certificate_serial_number: String,

    #[serde(rename = "SHA-256 Fingerprint")]
    pub sha256_fingerprint: String,

    #[serde(rename = "Trust Bits")]
    pub trust_bits: String,

    #[serde(rename = "Distrust for TLS After Date")]
    pub distrust_for_tls_after_date: String,

    #[serde(rename = "Mozilla Applied Constraints")]
    pub mozilla_applied_constraints: String,

    #[serde(rename = "PEM Info")]
    pub pem_info: String,
}

impl CertificateMetadata {
    /// Returns true iff the certificate has valid TrustBits that include TrustBits::Websites,
    /// and the certificate has no distrust for TLS after date, or has a valid distrust
    /// for TLS after date that is in the future compared to `now`. In all other cases this function
    /// returns false.
    fn trusted_for_tls(&self, now: &NaiveDate) -> bool {
        let has_tls_trust_bit = self.trust_bits().contains(&TrustBits::Websites);

        match (has_tls_trust_bit, self.tls_distrust_after()) {
            // No website trust bit - not trusted for tls.
            (false, _) => false,
            // Has website trust bit, no distrust after - trusted for tls.
            (true, None) => true,
            // Trust bit, populated distrust after - need to check date to decide.
            (true, Some(tls_distrust_after)) => {
                match now.cmp(&tls_distrust_after).is_ge() {
                    // We're past the distrust date - skip.
                    true => false,
                    // We haven't yet reached the distrust date - include.
                    false => true,
                }
            }
        }
    }

    /// Return the Mozilla applied constraints for the certificate (if any). The constraints
    /// will be encoded in the DER form expected by the webpki crate's TrustAnchor representation.
    fn mozilla_applied_constraints(&self) -> Option<Vec<u8>> {
        if self.mozilla_applied_constraints.is_empty() {
            return None;
        }

        // NOTE: To date there's only one CA with a applied constraints value, and it has only one
        // permitted subtree constraint imposed. It's not clear how multiple constraints would be
        // expressed. This method makes a best guess but may need to be revisited in the future.
        // https://groups.google.com/a/ccadb.org/g/public/c/TlDivISPVT4/m/jbWGuM4YAgAJ
        let included_subtrees = self.mozilla_applied_constraints.split(',');

        // Important: the webpki representation of name constraints elides:
        //   - the outer BITSTRING of the X.509 extension value.
        //   - the outer NameConstraints SEQUENCE over the permitted/excluded subtrees.
        //
        // See https://www.rfc-editor.org/rfc/rfc5280#section-4.2.1.10
        let der = yasna::construct_der(|writer| {
            // permittedSubtrees [0]
            writer.write_tagged_implicit(yasna::Tag::context(0), |writer| {
                // GeneralSubtrees
                writer.write_sequence(|writer| {
                    for included_subtree in included_subtrees {
                        // base GeneralName
                        writer.next().write_sequence(|writer| {
                            writer
                                .next()
                                // DnsName
                                .write_tagged_implicit(yasna::Tag::context(2), |writer| {
                                    writer
                                        .write_ia5_string(included_subtree.trim_start_matches('*'))
                                })
                        })
                        // minimum [0] (absent, 0 default)
                        // maximum [1] (must be omitted).
                    }
                })
            })
        });

        Some(der)
    }

    /// Return the NaiveDate after which this certificate should not be trusted for TLS (if any).
    /// Panics if there is a distrust for TLS after date value that can not be parsed.
    fn tls_distrust_after(&self) -> Option<NaiveDate> {
        match &self.distrust_for_tls_after_date {
            date if date.is_empty() => None,
            date => Some(
                NaiveDate::parse_from_str(date, "%Y.%m.%d")
                    .unwrap_or_else(|_| panic!("invalid distrust for tls after date: {:?}", date)),
            ),
        }
    }

    /// Returns the DER encoding of the certificate contained in the metadata PEM. Panics if
    /// there is an error, or no certificate in the PEM content.
    fn der(&self) -> Vec<u8> {
        let certs = rustls_pemfile::certs(&mut self.pem().as_bytes()).expect("invalid PEM");
        if certs.len() > 1 {
            panic!("more than one certificate in metadata PEM");
        }
        certs
            .first()
            .expect("missing certificate in metadata PEM")
            .clone()
    }

    /// Returns the serial number for the certificate. Panics if the certificate serial number
    /// from the metadata can not be parsed as a base 16 unsigned big integer.
    pub fn serial(&self) -> BigUint {
        BigUint::parse_bytes(self.certificate_serial_number.as_bytes(), 16)
            .expect("invalid certificate serial number")
    }

    /// Returns the colon separated string with the metadata SHA256 fingerprint for the
    /// certificate. Panics if the sha256 fingerprint from the metadata can't be decoded.
    pub fn sha256_fp(&self) -> String {
        x509_parser::utils::format_serial(
            &hex::decode(&self.sha256_fingerprint).expect("invalid sha256 fingerprint"),
        )
    }

    /// Returns the set of trust bits expressed for this certificate. Panics if the raw
    /// trust bits are invalid/unknown.
    fn trust_bits(&self) -> HashSet<TrustBits> {
        self.trust_bits.split(';').map(TrustBits::from).collect()
    }

    /// Returns the PEM metadata for the certificate with the leading/trailing single quotes
    /// removed.
    fn pem(&self) -> &str {
        self.pem_info.as_str().trim_matches('\'')
    }
}

impl PartialOrd for CertificateMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.sha256_fingerprint.cmp(&other.sha256_fingerprint))
    }
}

impl Ord for CertificateMetadata {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sha256_fingerprint.cmp(&other.sha256_fingerprint)
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
#[non_exhaustive]
/// TrustBits describe the possible Mozilla root certificate trust bits.
pub enum TrustBits {
    /// certificate is trusted for Websites (e.g. TLS).
    Websites,
    /// certificate is trusted for Email (e.g. S/MIME).
    Email,
}

impl From<&str> for TrustBits {
    fn from(value: &str) -> Self {
        match value {
            "Websites" => TrustBits::Websites,
            "Email" => TrustBits::Email,
            val => panic!("unknown trust bit: {:?}", val),
        }
    }
}

const HEADER: &str = r#"//!
//! This library is automatically generated from the Mozilla 
//! IncludedCACertificateReportPEMCSV report via ccadb.org. Don't edit it.
//!
//! The generation is done deterministically so you can verify it
//! yourself by inspecting and re-running the generation process.
//!

#![forbid(unsafe_code, unstable_features)]
#![deny(
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_extern_crates,
    unused_qualifications
)]

/// A trust anchor (sometimes called a root) for validating X.509 certificates
pub struct TrustAnchor<'a> {
    pub subject: &'a [u8],
    pub spki: &'a [u8],
    pub name_constraints: Option<&'a [u8]>,
}

"#;
