// Copyright 2015 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use crate::der::Tag;
use crate::{der, signed_data, Error};

pub enum EndEntityOrCa<'a> {
    EndEntity,
    Ca(&'a Cert<'a>),
}

pub struct Cert<'a> {
    pub ee_or_ca: EndEntityOrCa<'a>,

    pub signed_data: signed_data::SignedData<'a>,
    pub issuer: untrusted::Input<'a>,
    pub validity: untrusted::Input<'a>,
    pub subject: untrusted::Input<'a>,
    pub spki: der::Value<'a>,

    pub basic_constraints: Option<untrusted::Input<'a>>,
    pub eku: Option<untrusted::Input<'a>>,
    pub name_constraints: Option<untrusted::Input<'a>>,
    pub subject_alt_name: Option<untrusted::Input<'a>>,
}

pub(crate) fn parse_cert<'a>(
    cert_der: untrusted::Input<'a>,
    ee_or_ca: EndEntityOrCa<'a>,
) -> Result<Cert<'a>, Error> {
    let (tbs, signed_data) = cert_der.read_all(Error::BadDer, |cert_der| {
        der::nested(
            cert_der,
            der::Tag::Sequence,
            Error::BadDer,
            signed_data::parse_signed_data,
        )
    })?;

    tbs.read_all(Error::BadDer, |tbs| {
        version3(tbs)?;
        lenient_certificate_serial_number(tbs)?;

        let signature = der::expect_tag_and_get_value(tbs, der::Tag::Sequence)?;
        // TODO: In mozilla::pkix, the comparison is done based on the
        // normalized value (ignoring whether or not there is an optional NULL
        // parameter for RSA-based algorithms), so this may be too strict.
        if signature != signed_data.algorithm {
            return Err(Error::SignatureAlgorithmMismatch);
        }

        let issuer = der::expect_tag_and_get_value(tbs, der::Tag::Sequence)?;
        let validity = der::expect_tag_and_get_value(tbs, der::Tag::Sequence)?;
        let subject = der::expect_tag_and_get_value(tbs, der::Tag::Sequence)?;
        let spki = der::expect_tag(tbs, der::Tag::Sequence)?;

        // In theory there could be fields [1] issuerUniqueID and [2]
        // subjectUniqueID, but in practice there never are, and to keep the
        // code small and simple we don't accept any certificates that do
        // contain them.

        let mut cert = Cert {
            ee_or_ca,

            signed_data,
            issuer,
            validity,
            subject,
            spki,

            basic_constraints: None,
            eku: None,
            name_constraints: None,
            subject_alt_name: None,
        };

        if !tbs.at_end() {
            der::nested(
                tbs,
                der::Tag::ContextSpecificConstructed3,
                Error::MalformedExtensions,
                |tagged| {
                    der::nested_of_mut(
                        tagged,
                        der::Tag::Sequence,
                        der::Tag::Sequence,
                        Error::BadDer,
                        |extension| {
                            let extn_id = der::expect_tag_and_get_value(extension, der::Tag::OID)?;
                            let critical = der::optional_boolean(extension)?;
                            let extn_value =
                                der::expect_tag_and_get_value(extension, der::Tag::OctetString)?;
                            match remember_extension(&mut cert, extn_id, extn_value)? {
                                Understood::No if critical => {
                                    Err(Error::UnsupportedCriticalExtension)
                                }
                                _ => Ok(()),
                            }
                        },
                    )
                },
            )?;
        }

        Ok(cert)
    })
}

// mozilla::pkix supports v1, v2, v3, and v4, including both the implicit
// (correct) and explicit (incorrect) encoding of v1. We allow only v3.
fn version3(input: &mut untrusted::Reader) -> Result<(), Error> {
    der::nested(
        input,
        der::Tag::ContextSpecificConstructed0,
        Error::UnsupportedCertVersion,
        |input| {
            let version = der::small_nonnegative_integer(input)?;
            if version != 2 {
                // v3
                return Err(Error::UnsupportedCertVersion);
            }
            Ok(())
        },
    )
}

pub(crate) fn lenient_certificate_serial_number(
    input: &mut untrusted::Reader,
) -> Result<(), Error> {
    // https://tools.ietf.org/html/rfc5280#section-4.1.2.2:
    // * Conforming CAs MUST NOT use serialNumber values longer than 20 octets."
    // * "The serial number MUST be a positive integer [...]"
    //
    // However, we don't enforce these constraints, as there are widely-deployed trust anchors
    // and many X.509 implementations in common use that violate these constraints. This is called
    // out by the same section of RFC 5280 as cited above:
    //   Note: Non-conforming CAs may issue certificates with serial numbers
    //   that are negative or zero.  Certificate users SHOULD be prepared to
    //   gracefully handle such certificates.
    der::expect_tag_and_get_value(input, Tag::Integer)?;
    Ok(())
}

enum Understood {
    Yes,
    No,
}

fn remember_extension<'a>(
    cert: &mut Cert<'a>,
    extn_id: untrusted::Input,
    value: untrusted::Input<'a>,
) -> Result<Understood, Error> {
    // We don't do anything with certificate policies so we can safely ignore
    // all policy-related stuff. We assume that the policy-related extensions
    // are not marked critical.

    // id-ce 2.5.29
    static ID_CE: [u8; 2] = oid![2, 5, 29];

    if extn_id.len() != ID_CE.len() + 1 || !extn_id.as_slice_less_safe().starts_with(&ID_CE) {
        return Ok(Understood::No);
    }

    let out = match *extn_id.as_slice_less_safe().last().unwrap() {
        // id-ce-keyUsage 2.5.29.15. We ignore the KeyUsage extension. For CA
        // certificates, BasicConstraints.cA makes KeyUsage redundant. Firefox
        // and other common browsers do not check KeyUsage for end-entities,
        // though it would be kind of nice to ensure that a KeyUsage without
        // the keyEncipherment bit could not be used for RSA key exchange.
        15 => {
            return Ok(Understood::Yes);
        }

        // id-ce-subjectAltName 2.5.29.17
        17 => &mut cert.subject_alt_name,

        // id-ce-basicConstraints 2.5.29.19
        19 => &mut cert.basic_constraints,

        // id-ce-nameConstraints 2.5.29.30
        30 => &mut cert.name_constraints,

        // id-ce-extKeyUsage 2.5.29.37
        37 => &mut cert.eku,

        _ => {
            return Ok(Understood::No);
        }
    };

    match *out {
        Some(..) => {
            // The certificate contains more than one instance of this
            // extension.
            return Err(Error::ExtensionValueInvalid);
        }
        None => {
            // All the extensions that we care about are wrapped in a SEQUENCE.
            let sequence_value = value.read_all(Error::BadDer, |value| {
                der::expect_tag_and_get_value(value, der::Tag::Sequence)
            })?;
            *out = Some(sequence_value);
        }
    }

    Ok(Understood::Yes)
}
