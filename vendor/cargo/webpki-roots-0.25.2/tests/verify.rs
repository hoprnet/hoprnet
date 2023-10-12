use std::convert::TryFrom;

use webpki::{EndEntityCert, KeyUsage, SubjectNameRef, Time};
use webpki_roots::TLS_SERVER_ROOTS;

#[test]
fn tubitak_name_constraint_works() {
    let root = include_bytes!("data/tubitak/root.der");
    let inter = include_bytes!("data/tubitak/inter.der");
    let subj = include_bytes!("data/tubitak/subj.der");

    let roots = TLS_SERVER_ROOTS
        .iter()
        .map(|ta| webpki::TrustAnchor {
            subject: ta.subject,
            spki: ta.spki,
            name_constraints: ta.name_constraints,
        })
        .collect::<Vec<_>>();

    let now = Time::from_seconds_since_unix_epoch(1493668479);
    let cert = EndEntityCert::try_from(&subj[..]).unwrap();
    cert.verify_for_usage(
        ALL_ALGORITHMS,
        &roots,
        &[&inter[..], &root[..]],
        now,
        KeyUsage::server_auth(),
        &[],
    )
    .unwrap();

    let subject = SubjectNameRef::try_from_ascii_str("testssl.kamusm.gov.tr").unwrap();
    cert.verify_is_valid_for_subject_name(subject).unwrap();
}

static ALL_ALGORITHMS: &[&webpki::SignatureAlgorithm] = &[
    &webpki::ECDSA_P256_SHA256,
    &webpki::ECDSA_P256_SHA384,
    &webpki::ECDSA_P384_SHA256,
    &webpki::ECDSA_P384_SHA384,
    &webpki::RSA_PKCS1_2048_8192_SHA256,
    &webpki::RSA_PKCS1_2048_8192_SHA384,
    &webpki::RSA_PKCS1_2048_8192_SHA512,
    &webpki::RSA_PKCS1_3072_8192_SHA384,
    &webpki::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
    &webpki::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
    &webpki::RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
];
