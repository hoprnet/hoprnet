use std::ascii::escape_default;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::io::Cursor;

use ring::digest;

#[tokio::test]
async fn generated_code_is_fresh() {
    // Fetch the list of certificates as a PEM file from mkcert.org

    let mut except = String::with_capacity(128);
    for (i, ca) in EXCLUDED_CAS.iter().enumerate() {
        if i > 0 {
            except.push('+');
        }

        except
            .write_fmt(format_args!(
                "{}",
                percent_encoding::percent_encode(ca.as_bytes(), percent_encoding::NON_ALPHANUMERIC,)
            ))
            .unwrap();
    }

    let url = format!("https://mkcert.org/generate/all/except/{except}");
    eprintln!("fetching {url}...");
    let body = reqwest::get(&url).await.unwrap().text().await.unwrap();
    fs::write("fetched.pem", &body).unwrap();

    // Split file contents into a Vec<Vec<&str>> where each inner Vec represents one certificate

    let mut certs = Vec::with_capacity(64);
    let mut current = Vec::with_capacity(16);
    for ln in body.lines() {
        if !ln.is_empty() {
            current.push(ln);
            continue;
        } else if current.is_empty() {
            continue;
        }

        certs.push(current);
        current = Vec::with_capacity(16);
    }

    certs.push(current);

    // Parse each certificate and check fingerprints

    let mut hashed = HashMap::<Vec<u8>, (Vec<&str>, Vec<u8>)>::default();
    for cert in certs {
        let start = cert
            .iter()
            .position(|&ln| ln == "-----BEGIN CERTIFICATE-----")
            .unwrap();

        // Parse PEM to get the DER.
        let pem = cert[start..].join("\n");
        let mut reader = Cursor::new(pem.as_bytes());
        let der = match rustls_pemfile::read_one(&mut reader).unwrap().unwrap() {
            rustls_pemfile::Item::X509Certificate(der) => der,
            _ => unreachable!(),
        };

        // Check if our hash matches the one in the file.
        let their_fingerprint = cert
            .iter()
            .find_map(|&ln| ln.strip_prefix("# SHA256 Fingerprint: "))
            .unwrap();
        let hash = digest::digest(&digest::SHA256, &der);
        assert!(
            !hashed.contains_key(hash.as_ref()),
            "duplicate hash: {:#?}",
            &cert
        );

        let our_fingerprint = hash
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        assert_eq!(
            their_fingerprint.replace(':', ""),
            our_fingerprint,
            "{:#?}",
            &cert
        );

        hashed.insert(hash.as_ref().to_vec(), (cert, der));
    }

    // For the given certificate subject name, store a name constraints encoding
    // which will be applied to that certificate. This data is sourced from
    // https://hg.mozilla.org/projects/nss/file/tip/lib/certdb/genname.c such that
    // webpki-roots implements the same policy in this respect as the Mozilla root program.
    let mut imposed_constraints = HashMap::<Vec<u8>, Vec<u8>>::default();
    imposed_constraints.insert(
        concat(TUBITAK1_SUBJECT_DN),
        TUBITAK1_NAME_CONSTRAINTS.to_vec(),
    );

    // Generate the trust anchors, sorted by fingerprint

    let mut hashes = hashed.into_iter().collect::<Vec<_>>();
    hashes.sort_by(|a, b| a.0.cmp(&b.0));

    let (mut subject, mut spki, mut name_constraints) =
        (String::new(), String::new(), String::new());
    let mut code = String::with_capacity(256 * 1_024);
    code.push_str(HEADER);
    code.push_str("pub const TLS_SERVER_ROOTS: &[TrustAnchor] = &[\n");
    for (_, (lines, der)) in hashes {
        let ta = webpki::TrustAnchor::try_from_cert_der(&der).unwrap();
        subject.clear();
        for &b in ta.subject {
            write!(&mut subject, "{}", escape_default(b)).unwrap();
        }

        spki.clear();
        for &b in ta.spki {
            write!(&mut spki, "{}", escape_default(b)).unwrap();
        }

        name_constraints.clear();
        let nc = imposed_constraints
            .get(ta.subject)
            .map(|nc| nc.as_slice())
            .or(ta.name_constraints);
        if let Some(nc) = nc {
            for &b in nc {
                write!(&mut name_constraints, "{}", escape_default(b)).unwrap();
            }
        }

        // Write comment with source

        code.push_str("  /*\n");
        for &ln in lines.iter() {
            code.push_str("   * ");
            match ln.strip_prefix("# ") {
                Some(ln) => code.push_str(ln),
                None => code.push_str(ln),
            }
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

fn concat(parts: &[&[u8]]) -> Vec<u8> {
    let mut v = Vec::with_capacity(128);
    for &b in parts {
        v.extend(b);
    }
    v
}

// TUBITAK Kamu SM SSL Kok Sertifikasi - Surum 1
const TUBITAK1_SUBJECT_DN: &[&[u8]] = &[
    b"\x31\x0b\x30\x09\x06\x03\x55\x04\x06\x13\x02",
    b"TR",
    b"\x31\x18\x30\x16\x06\x03\x55\x04\x07\x13\x0f",
    b"Gebze - Kocaeli",
    b"\x31\x42\x30\x40\x06\x03\x55\x04\x0a\x13\x39",
    b"Turkiye Bilimsel ve Teknolojik Arastirma Kurumu - TUBITAK",
    b"\x31\x2d\x30\x2b\x06\x03\x55\x04\x0b\x13\x24",
    b"Kamu Sertifikasyon Merkezi - Kamu SM",
    b"\x31\x36\x30\x34\x06\x03\x55\x04\x03\x13\x2d",
    b"TUBITAK Kamu SM SSL Kok Sertifikasi - Surum 1",
];

const TUBITAK1_NAME_CONSTRAINTS: &[u8] =
    &[0xA0, 0x07, 0x30, 0x05, 0x82, 0x03, 0x2E, 0x74, 0x72];

const EXCLUDED_CAS: &[&str] = &[
    // See https://bugzilla.mozilla.org/show_bug.cgi?id=1266574.
    "Buypass Class 2 CA 1",
    // https://blog.mozilla.org/security/2015/04/02/distrusting-new-cnnic-certificates/
    // https://security.googleblog.com/2015/03/maintaining-digital-certificate-security.html
    "China Internet Network Information Center",
    "CNNIC",
    // See https://bugzilla.mozilla.org/show_bug.cgi?id=1283326.
    "RSA Security 2048 v3",
    // https://bugzilla.mozilla.org/show_bug.cgi?id=1272158
    "Root CA Generalitat Valenciana",
    // See https://wiki.mozilla.org/CA:WoSign_Issues.
    "StartCom",
    "WoSign",
    // See https://cabforum.org/pipermail/public/2016-September/008475.html.
    // Both the ASCII and non-ASCII names are required.
    "TÜRKTRUST",
    "TURKTRUST",
];

const HEADER: &str = r#"//!
//! This library is automatically generated from the Mozilla certificate
//! store via mkcert.org.  Don't edit it.
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
