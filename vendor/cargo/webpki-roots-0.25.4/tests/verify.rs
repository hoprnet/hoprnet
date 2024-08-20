use std::convert::TryFrom;

use rcgen::{BasicConstraints, Certificate, CertificateParams, DnType, IsCa, KeyUsagePurpose};
use webpki::{EndEntityCert, Error, KeyUsage, SubjectNameRef, Time};
use x509_parser::extensions::{GeneralName, NameConstraints as X509ParserNameConstraints};
use x509_parser::prelude::FromDer;

use webpki_roots::TLS_SERVER_ROOTS;

#[test]
fn name_constraints() {
    for name_constraints in TLS_SERVER_ROOTS.iter().filter_map(|ta| ta.name_constraints) {
        let time = Time::from_seconds_since_unix_epoch(0x40000000); // Time matching rcgen default.
        let test_case = ConstraintTest::new(name_constraints);
        let trust_anchors =
            &[webpki::TrustAnchor::try_from_cert_der(&test_case.trust_anchor).unwrap()];

        // Each permitted EE should verify without error.
        for permitted_ee in test_case.permitted_certs {
            webpki::EndEntityCert::try_from(permitted_ee.as_slice())
                .unwrap()
                .verify_for_usage(
                    ALL_ALGORITHMS,
                    trust_anchors,
                    &[],
                    time,
                    KeyUsage::server_auth(),
                    &[],
                )
                .unwrap();
        }

        // Each forbidden EE should fail to verify with the expected name constraint error.
        for forbidden_ee in test_case.forbidden_certs {
            let result = webpki::EndEntityCert::try_from(forbidden_ee.as_slice())
                .unwrap()
                .verify_for_usage(
                    ALL_ALGORITHMS,
                    trust_anchors,
                    &[],
                    time,
                    KeyUsage::server_auth(),
                    &[],
                );
            assert!(matches!(result, Err(Error::NameConstraintViolation)));
        }
    }
}

struct ConstraintTest {
    trust_anchor: Vec<u8>,
    permitted_certs: Vec<Vec<u8>>,
    forbidden_certs: Vec<Vec<u8>>,
}

impl ConstraintTest {
    fn new(webpki_name_constraints: &[u8]) -> Self {
        // Create a trust anchor CA certificate that has the name constraints we want to test.
        let mut trust_anchor = CertificateParams::new([]);
        trust_anchor
            .distinguished_name
            .push(DnType::CommonName, "Name Constraint Test CA");
        trust_anchor.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        trust_anchor.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::DigitalSignature,
        ];
        let name_constraints = rcgen_name_constraints(webpki_name_constraints);
        trust_anchor.name_constraints = Some(name_constraints.clone());
        let trust_anchor = Certificate::from_params(trust_anchor).unwrap();

        let certs_for_subtrees = |suffix| -> Vec<Vec<u8>> {
            name_constraints
                .permitted_subtrees
                .iter()
                .filter_map(|subtree| match subtree {
                    rcgen::GeneralSubtree::DnsName(dns_name) => Some(rcgen_ee_for_name(
                        format!("valid{}{}", dns_name, suffix),
                        &trust_anchor,
                    )),
                    _ => None,
                })
                .collect()
        };

        Self {
            trust_anchor: trust_anchor.serialize_der().unwrap(),
            // For each permitted subtree in the name constraints, issue an end entity certificate
            // that contains a DNS name matching the permitted subtree base.
            permitted_certs: certs_for_subtrees(""),
            // For each permitted subtree in the name constraints, issue an end entity certificate
            // that contains a DNS name that will **not** match the permitted subtree base.
            forbidden_certs: certs_for_subtrees(".invalid"),
        }
    }
}

fn rcgen_ee_for_name(name: String, issuer: &Certificate) -> Vec<u8> {
    let mut ee = CertificateParams::new(vec![name.clone()]);
    ee.distinguished_name.push(DnType::CommonName, name);
    ee.is_ca = IsCa::NoCa;
    Certificate::from_params(ee)
        .unwrap()
        .serialize_der_with_signer(issuer)
        .unwrap()
}

/// Convert the webpki trust anchor DER encoding of name constraints to rcgen NameConstraints.
fn rcgen_name_constraints(der: &[u8]) -> rcgen::NameConstraints {
    // x509 parser expects the outer SEQUENCE that the webpki trust anchor representation elides
    // so wrap the DER up.
    let wrapped_der = yasna::construct_der(|writer| {
        writer.write_sequence(|writer| {
            writer.next().write_der(der);
        })
    });

    // Constraints should parse with no trailing data.
    let (trailing, constraints) = X509ParserNameConstraints::from_der(&wrapped_der).unwrap();
    assert!(
        trailing.is_empty(),
        "unexpected trailing DER in name constraint"
    );

    // There should be at least one permitted subtree.
    assert!(
        constraints.permitted_subtrees.is_some(),
        "empty permitted subtrees in constraints"
    );

    // We don't expect any excluded subtrees as this time.
    assert!(constraints.excluded_subtrees.is_none());

    // Collect all of the DNS names from the x509-parser representation, mapping to the rcgen
    // representation usable in cert parameters. We don't expect to find any other types of general
    // name and x509-parser doesn't parse the subtree minimum and maximum (which we would assert to
    // be missing for proper encoding anyway).
    let permitted_subtrees = match constraints.permitted_subtrees {
        None => Vec::default(),
        Some(subtrees) => subtrees
            .iter()
            .map(|subtree| match &subtree.base {
                GeneralName::DNSName(base) => rcgen::GeneralSubtree::DnsName(base.to_string()),
                name => panic!("unexpected subtree base general name type: {}", name),
            })
            .collect(),
    };

    rcgen::NameConstraints {
        permitted_subtrees,
        excluded_subtrees: Vec::default(),
    }
}

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
