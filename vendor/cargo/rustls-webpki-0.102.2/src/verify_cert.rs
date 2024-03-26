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

use core::default::Default;
use core::ops::ControlFlow;

use pki_types::{CertificateDer, SignatureVerificationAlgorithm, TrustAnchor, UnixTime};

use crate::cert::Cert;
use crate::crl::RevocationOptions;
use crate::der::{self, FromDer};
use crate::end_entity::EndEntityCert;
use crate::error::Error;
use crate::{public_values_eq, signed_data, subject_name};

// Use `'a` for lifetimes that we don't care about, `'p` for lifetimes that become a part of
// the `VerifiedPath`.
pub(crate) struct ChainOptions<'a, 'p> {
    pub(crate) eku: KeyUsage,
    pub(crate) supported_sig_algs: &'a [&'a dyn SignatureVerificationAlgorithm],
    pub(crate) trust_anchors: &'p [TrustAnchor<'p>],
    pub(crate) intermediate_certs: &'p [CertificateDer<'p>],
    pub(crate) revocation: Option<RevocationOptions<'a>>,
}

impl<'a, 'p: 'a> ChainOptions<'a, 'p> {
    pub(crate) fn build_chain(
        &self,
        end_entity: &'p EndEntityCert<'p>,
        time: UnixTime,
        verify_path: Option<&dyn Fn(&VerifiedPath<'_>) -> Result<(), Error>>,
    ) -> Result<VerifiedPath<'p>, Error> {
        let mut path = PartialPath::new(end_entity);
        match self.build_chain_inner(&mut path, time, verify_path, 0, &mut Budget::default()) {
            Ok(anchor) => Ok(VerifiedPath::new(end_entity, anchor, path)),
            Err(ControlFlow::Break(err)) | Err(ControlFlow::Continue(err)) => Err(err),
        }
    }

    fn build_chain_inner(
        &self,
        path: &mut PartialPath<'p>,
        time: UnixTime,
        verify_path: Option<&dyn Fn(&VerifiedPath<'_>) -> Result<(), Error>>,
        sub_ca_count: usize,
        budget: &mut Budget,
    ) -> Result<&'p TrustAnchor<'p>, ControlFlow<Error, Error>> {
        let role = path.node().role();

        check_issuer_independent_properties(path.head(), time, role, sub_ca_count, self.eku.inner)?;

        // TODO: HPKP checks.

        let result = loop_while_non_fatal_error(
            Error::UnknownIssuer,
            self.trust_anchors,
            |trust_anchor: &TrustAnchor| {
                let trust_anchor_subject = untrusted::Input::from(trust_anchor.subject.as_ref());
                if !public_values_eq(path.head().issuer, trust_anchor_subject) {
                    return Err(Error::UnknownIssuer.into());
                }

                // TODO: check_distrust(trust_anchor_subject, trust_anchor_spki)?;

                let node = path.node();
                self.check_signed_chain(&node, trust_anchor, budget)?;
                check_signed_chain_name_constraints(&node, trust_anchor, budget)?;

                let verify = match verify_path {
                    Some(verify) => verify,
                    None => return Ok(trust_anchor),
                };

                let candidate = VerifiedPath {
                    end_entity: path.end_entity,
                    intermediates: Intermediates::Borrowed(&path.intermediates[..path.used]),
                    anchor: trust_anchor,
                };

                match verify(&candidate) {
                    Ok(()) => Ok(trust_anchor),
                    Err(err) => Err(ControlFlow::Continue(err)),
                }
            },
        );

        let err = match result {
            Ok(anchor) => return Ok(anchor),
            // Fatal errors should halt further path building.
            res @ Err(ControlFlow::Break(_)) => return res,
            // Non-fatal errors should be carried forward as the default_error for subsequent
            // loop_while_non_fatal_error processing and only returned once all other path-building
            // options have been exhausted.
            Err(ControlFlow::Continue(err)) => err,
        };

        loop_while_non_fatal_error(err, self.intermediate_certs, |cert_der| {
            let potential_issuer = Cert::from_der(untrusted::Input::from(cert_der))?;
            if !public_values_eq(potential_issuer.subject, path.head().issuer) {
                return Err(Error::UnknownIssuer.into());
            }

            // Prevent loops; see RFC 4158 section 5.2.
            if path.node().iter().any(|prev| {
                public_values_eq(potential_issuer.spki, prev.cert.spki)
                    && public_values_eq(potential_issuer.subject, prev.cert.subject)
            }) {
                return Err(Error::UnknownIssuer.into());
            }

            let next_sub_ca_count = match role {
                Role::EndEntity => sub_ca_count,
                Role::Issuer => sub_ca_count + 1,
            };

            budget.consume_build_chain_call()?;
            path.push(potential_issuer)?;
            let result = self.build_chain_inner(path, time, verify_path, next_sub_ca_count, budget);
            if result.is_err() {
                path.pop();
            }

            result
        })
    }

    fn check_signed_chain(
        &self,
        path: &PathNode<'_>,
        trust_anchor: &TrustAnchor,
        budget: &mut Budget,
    ) -> Result<(), ControlFlow<Error, Error>> {
        let mut spki_value = untrusted::Input::from(trust_anchor.subject_public_key_info.as_ref());
        let mut issuer_subject = untrusted::Input::from(trust_anchor.subject.as_ref());
        let mut issuer_key_usage = None; // TODO(XXX): Consider whether to track TrustAnchor KU.
        for path in path.iter() {
            signed_data::verify_signed_data(
                self.supported_sig_algs,
                spki_value,
                &path.cert.signed_data,
                budget,
            )?;

            if let Some(revocation_opts) = &self.revocation {
                revocation_opts.check(
                    &path,
                    issuer_subject,
                    spki_value,
                    issuer_key_usage,
                    self.supported_sig_algs,
                    budget,
                )?;
            }

            spki_value = path.cert.spki;
            issuer_subject = path.cert.subject;
            issuer_key_usage = path.cert.key_usage;
        }

        Ok(())
    }
}

/// Path from end-entity certificate to trust anchor that's been verified.
///
/// See [`EndEntityCert::verify_for_usage()`] for more details on what verification entails.
pub struct VerifiedPath<'p> {
    end_entity: &'p EndEntityCert<'p>,
    intermediates: Intermediates<'p>,
    anchor: &'p TrustAnchor<'p>,
}

impl<'p> VerifiedPath<'p> {
    fn new(
        end_entity: &'p EndEntityCert<'p>,
        anchor: &'p TrustAnchor<'p>,
        partial: PartialPath<'p>,
    ) -> Self {
        Self {
            end_entity,
            intermediates: Intermediates::Owned {
                certs: partial.intermediates,
                used: partial.used,
            },
            anchor,
        }
    }

    /// Yields a (double-ended) iterator over the intermediate certificates in this path.
    pub fn intermediate_certificates(&'p self) -> IntermediateIterator<'p> {
        IntermediateIterator {
            intermediates: self.intermediates.as_ref(),
        }
    }

    /// Yields the end-entity certificate for this path.
    pub fn end_entity(&self) -> &'p EndEntityCert<'p> {
        self.end_entity
    }

    /// Yields the trust anchor for this path.
    pub fn anchor(&self) -> &'p TrustAnchor<'p> {
        self.anchor
    }
}

/// Iterator over a path's intermediate certificates.
///
/// Implements [`DoubleEndedIterator`] so it can be traversed in both directions.
pub struct IntermediateIterator<'a> {
    /// Invariant: all of these `Option`s are `Some`.
    intermediates: &'a [Option<Cert<'a>>],
}

impl<'a> Iterator for IntermediateIterator<'a> {
    type Item = &'a Cert<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.intermediates.split_first() {
            Some((head, tail)) => {
                self.intermediates = tail;
                Some(head.as_ref().unwrap())
            }
            None => None,
        }
    }
}

impl<'a> DoubleEndedIterator for IntermediateIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.intermediates.split_last() {
            Some((head, tail)) => {
                self.intermediates = tail;
                Some(head.as_ref().unwrap())
            }
            None => None,
        }
    }
}

#[allow(clippy::large_enum_variant)]
enum Intermediates<'a> {
    Owned {
        certs: [Option<Cert<'a>>; MAX_SUB_CA_COUNT],
        used: usize,
    },
    Borrowed(&'a [Option<Cert<'a>>]),
}

impl<'a> AsRef<[Option<Cert<'a>>]> for Intermediates<'a> {
    fn as_ref(&self) -> &[Option<Cert<'a>>] {
        match self {
            Intermediates::Owned { certs, used } => &certs[..*used],
            Intermediates::Borrowed(certs) => certs,
        }
    }
}

fn check_signed_chain_name_constraints(
    path: &PathNode<'_>,
    trust_anchor: &TrustAnchor,
    budget: &mut Budget,
) -> Result<(), ControlFlow<Error, Error>> {
    let mut name_constraints = trust_anchor
        .name_constraints
        .as_ref()
        .map(|der| untrusted::Input::from(der.as_ref()));

    for path in path.iter() {
        untrusted::read_all_optional(name_constraints, Error::BadDer, |value| {
            subject_name::check_name_constraints(value, &path, budget)
        })?;

        name_constraints = path.cert.name_constraints;
    }

    Ok(())
}

pub(crate) struct Budget {
    signatures: usize,
    build_chain_calls: usize,
    name_constraint_comparisons: usize,
}

impl Budget {
    #[inline]
    pub(crate) fn consume_signature(&mut self) -> Result<(), Error> {
        self.signatures = self
            .signatures
            .checked_sub(1)
            .ok_or(Error::MaximumSignatureChecksExceeded)?;
        Ok(())
    }

    #[inline]
    fn consume_build_chain_call(&mut self) -> Result<(), Error> {
        self.build_chain_calls = self
            .build_chain_calls
            .checked_sub(1)
            .ok_or(Error::MaximumPathBuildCallsExceeded)?;
        Ok(())
    }

    #[inline]
    pub(crate) fn consume_name_constraint_comparison(&mut self) -> Result<(), Error> {
        self.name_constraint_comparisons = self
            .name_constraint_comparisons
            .checked_sub(1)
            .ok_or(Error::MaximumNameConstraintComparisonsExceeded)?;
        Ok(())
    }
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            // This limit is taken from the remediation for golang CVE-2018-16875.  However,
            // note that golang subsequently implemented AKID matching due to this limit
            // being hit in real applications (see <https://github.com/spiffe/spire/issues/1004>).
            // So this may actually be too aggressive.
            signatures: 100,

            // This limit is taken from mozilla::pkix, see:
            // <https://github.com/nss-dev/nss/blob/bb4a1d38dd9e92923525ac6b5ed0288479f3f3fc/lib/mozpkix/lib/pkixbuild.cpp#L381-L393>
            build_chain_calls: 200_000,

            // This limit is taken from golang crypto/x509's default, see:
            // <https://github.com/golang/go/blob/ac17bb6f13979f2ab9fcd45f0758b43ed72d0973/src/crypto/x509/verify.go#L588-L592>
            name_constraint_comparisons: 250_000,
        }
    }
}

fn check_issuer_independent_properties(
    cert: &Cert,
    time: UnixTime,
    role: Role,
    sub_ca_count: usize,
    eku: ExtendedKeyUsage,
) -> Result<(), Error> {
    // TODO: check_distrust(trust_anchor_subject, trust_anchor_spki)?;
    // TODO: Check signature algorithm like mozilla::pkix.
    // TODO: Check SPKI like mozilla::pkix.
    // TODO: check for active distrust like mozilla::pkix.

    // For cert validation, we ignore the KeyUsage extension. For CA
    // certificates, BasicConstraints.cA makes KeyUsage redundant. Firefox
    // and other common browsers do not check KeyUsage for end-entities,
    // though it would be kind of nice to ensure that a KeyUsage without
    // the keyEncipherment bit could not be used for RSA key exchange.

    cert.validity
        .read_all(Error::BadDer, |value| check_validity(value, time))?;
    untrusted::read_all_optional(cert.basic_constraints, Error::BadDer, |value| {
        check_basic_constraints(value, role, sub_ca_count)
    })?;
    untrusted::read_all_optional(cert.eku, Error::BadDer, |value| eku.check(value))?;

    Ok(())
}

// https://tools.ietf.org/html/rfc5280#section-4.1.2.5
fn check_validity(input: &mut untrusted::Reader, time: UnixTime) -> Result<(), Error> {
    let not_before = UnixTime::from_der(input)?;
    let not_after = UnixTime::from_der(input)?;

    if not_before > not_after {
        return Err(Error::InvalidCertValidity);
    }
    if time < not_before {
        return Err(Error::CertNotValidYet);
    }
    if time > not_after {
        return Err(Error::CertExpired);
    }

    // TODO: mozilla::pkix allows the TrustDomain to check not_before and
    // not_after, to enforce things like a maximum validity period. We should
    // do something similar.

    Ok(())
}

// https://tools.ietf.org/html/rfc5280#section-4.2.1.9
fn check_basic_constraints(
    input: Option<&mut untrusted::Reader>,
    role: Role,
    sub_ca_count: usize,
) -> Result<(), Error> {
    let (is_ca, path_len_constraint) = match input {
        Some(input) => {
            let is_ca = bool::from_der(input)?;

            // https://bugzilla.mozilla.org/show_bug.cgi?id=985025: RFC 5280
            // says that a certificate must not have pathLenConstraint unless
            // it is a CA certificate, but some real-world end-entity
            // certificates have pathLenConstraint.
            let path_len_constraint = if !input.at_end() {
                Some(usize::from(u8::from_der(input)?))
            } else {
                None
            };

            (is_ca, path_len_constraint)
        }
        None => (false, None),
    };

    match (role, is_ca, path_len_constraint) {
        (Role::EndEntity, true, _) => Err(Error::CaUsedAsEndEntity),
        (Role::Issuer, false, _) => Err(Error::EndEntityUsedAsCa),
        (Role::Issuer, true, Some(len)) if sub_ca_count > len => {
            Err(Error::PathLenConstraintViolated)
        }
        _ => Ok(()),
    }
}

/// The expected key usage of a certificate.
///
/// This type represents the expected key usage of an end entity certificate. Although for most
/// kinds of certificates the extended key usage extension is optional (and so certificates
/// not carrying a particular value in the EKU extension are acceptable). If the extension
/// is present, the certificate MUST only be used for one of the purposes indicated.
///
/// <https://www.rfc-editor.org/rfc/rfc5280#section-4.2.1.12>
#[derive(Clone, Copy)]
pub struct KeyUsage {
    inner: ExtendedKeyUsage,
}

impl KeyUsage {
    /// Construct a new [`KeyUsage`] as appropriate for server certificate authentication.
    ///
    /// As specified in <https://www.rfc-editor.org/rfc/rfc5280#section-4.2.1.12>, this does not require the certificate to specify the eKU extension.
    pub const fn server_auth() -> Self {
        Self {
            inner: ExtendedKeyUsage::RequiredIfPresent(EKU_SERVER_AUTH),
        }
    }

    /// Construct a new [`KeyUsage`] as appropriate for client certificate authentication.
    ///
    /// As specified in <>, this does not require the certificate to specify the eKU extension.
    pub const fn client_auth() -> Self {
        Self {
            inner: ExtendedKeyUsage::RequiredIfPresent(EKU_CLIENT_AUTH),
        }
    }

    /// Construct a new [`KeyUsage`] requiring a certificate to support the specified OID.
    pub const fn required(oid: &'static [u8]) -> Self {
        Self {
            inner: ExtendedKeyUsage::Required(KeyPurposeId::new(oid)),
        }
    }
}

/// Extended Key Usage (EKU) of a certificate.
#[derive(Clone, Copy)]
enum ExtendedKeyUsage {
    /// The certificate must contain the specified [`KeyPurposeId`] as EKU.
    Required(KeyPurposeId),

    /// If the certificate has EKUs, then the specified [`KeyPurposeId`] must be included.
    RequiredIfPresent(KeyPurposeId),
}

impl ExtendedKeyUsage {
    // https://tools.ietf.org/html/rfc5280#section-4.2.1.12
    fn check(&self, input: Option<&mut untrusted::Reader>) -> Result<(), Error> {
        let input = match (input, self) {
            (Some(input), _) => input,
            (None, Self::RequiredIfPresent(_)) => return Ok(()),
            (None, Self::Required(_)) => return Err(Error::RequiredEkuNotFound),
        };

        loop {
            let value = der::expect_tag(input, der::Tag::OID)?;
            if self.key_purpose_id_equals(value) {
                input.skip_to_end();
                break;
            }

            if input.at_end() {
                return Err(Error::RequiredEkuNotFound);
            }
        }

        Ok(())
    }

    fn key_purpose_id_equals(&self, value: untrusted::Input<'_>) -> bool {
        public_values_eq(
            match self {
                ExtendedKeyUsage::Required(eku) => *eku,
                ExtendedKeyUsage::RequiredIfPresent(eku) => *eku,
            }
            .oid_value,
            value,
        )
    }
}

/// An OID value indicating an Extended Key Usage (EKU) key purpose.
#[derive(Clone, Copy)]
struct KeyPurposeId {
    oid_value: untrusted::Input<'static>,
}

impl KeyPurposeId {
    /// Construct a new [`KeyPurposeId`].
    ///
    /// `oid` is the OBJECT IDENTIFIER in bytes.
    const fn new(oid: &'static [u8]) -> Self {
        Self {
            oid_value: untrusted::Input::from(oid),
        }
    }
}

impl PartialEq<Self> for KeyPurposeId {
    fn eq(&self, other: &Self) -> bool {
        public_values_eq(self.oid_value, other.oid_value)
    }
}

impl Eq for KeyPurposeId {}

// id-pkix            OBJECT IDENTIFIER ::= { 1 3 6 1 5 5 7 }
// id-kp              OBJECT IDENTIFIER ::= { id-pkix 3 }

// id-kp-serverAuth   OBJECT IDENTIFIER ::= { id-kp 1 }
const EKU_SERVER_AUTH: KeyPurposeId = KeyPurposeId::new(&oid!(1, 3, 6, 1, 5, 5, 7, 3, 1));

// id-kp-clientAuth   OBJECT IDENTIFIER ::= { id-kp 2 }
const EKU_CLIENT_AUTH: KeyPurposeId = KeyPurposeId::new(&oid!(1, 3, 6, 1, 5, 5, 7, 3, 2));

fn loop_while_non_fatal_error<'a, V: 'a>(
    default_error: Error,
    values: V,
    mut f: impl FnMut(V::Item) -> Result<&'a TrustAnchor<'a>, ControlFlow<Error, Error>>,
) -> Result<&'a TrustAnchor<'a>, ControlFlow<Error, Error>>
where
    V: IntoIterator,
{
    let mut error = default_error;
    for v in values {
        match f(v) {
            Ok(anchor) => return Ok(anchor),
            // Fatal errors should halt further looping.
            res @ Err(ControlFlow::Break(_)) => return res,
            // Non-fatal errors should be ranked by specificity and only returned
            // once all other path-building options have been exhausted.
            Err(ControlFlow::Continue(new_error)) => error = error.most_specific(new_error),
        }
    }
    Err(error.into())
}

/// A path for consideration in path building.
///
/// This represents a partial path because it does not yet contain the trust anchor. It stores
/// the end-entity certificates, and an array of intermediate certificates.
pub(crate) struct PartialPath<'a> {
    end_entity: &'a EndEntityCert<'a>,
    /// Intermediate certificates, in order from end-entity to trust anchor.
    ///
    /// Invariant: all values below `used` are `Some`.
    intermediates: [Option<Cert<'a>>; MAX_SUB_CA_COUNT],
    /// The number of `Some` values in `intermediates`.
    ///
    /// The next `Cert` passed to `push()` will be placed at `intermediates[used]`.
    /// If this value is 0, the path contains only the end-entity certificate.
    used: usize,
}

impl<'a> PartialPath<'a> {
    pub(crate) fn new(end_entity: &'a EndEntityCert<'a>) -> Self {
        Self {
            end_entity,
            intermediates: Default::default(),
            used: 0,
        }
    }

    pub(crate) fn push(&mut self, cert: Cert<'a>) -> Result<(), ControlFlow<Error, Error>> {
        if self.used >= MAX_SUB_CA_COUNT {
            return Err(Error::MaximumPathDepthExceeded.into());
        }

        self.intermediates[self.used] = Some(cert);
        self.used += 1;
        Ok(())
    }

    fn pop(&mut self) {
        debug_assert!(self.used > 0);
        if self.used == 0 {
            return;
        }

        self.used -= 1;
        self.intermediates[self.used] = None;
    }

    pub(crate) fn node(&self) -> PathNode<'_> {
        PathNode {
            path: self,
            index: self.used,
            cert: self.head(),
        }
    }

    /// Current head of the path.
    pub(crate) fn head(&self) -> &Cert<'a> {
        self.get(self.used)
    }

    /// Get the certificate at index `idx` in the path.
    ///
    // `idx` must be in the range `0..=self.used`; `idx` 0 thus yields the `end_entity`,
    // while subsequent indexes yield the intermediate at `self.intermediates[idx - 1]`.
    fn get(&self, idx: usize) -> &Cert<'a> {
        match idx {
            0 => self.end_entity,
            _ => self.intermediates[idx - 1].as_ref().unwrap(),
        }
    }
}

const MAX_SUB_CA_COUNT: usize = 6;

pub(crate) struct PathNode<'a> {
    /// The path we're iterating.
    path: &'a PartialPath<'a>,
    /// The index of the current node in the path (input for `path.get()`).
    index: usize,
    /// The [`Cert`] at `index`.
    pub(crate) cert: &'a Cert<'a>,
}

impl<'a> PathNode<'a> {
    pub(crate) fn iter(&self) -> PathIter<'a> {
        PathIter {
            path: self.path,
            next: Some(self.index),
        }
    }

    pub(crate) fn role(&self) -> Role {
        match self.index {
            0 => Role::EndEntity,
            _ => Role::Issuer,
        }
    }
}

pub(crate) struct PathIter<'a> {
    path: &'a PartialPath<'a>,
    next: Option<usize>,
}

impl<'a> Iterator for PathIter<'a> {
    type Item = PathNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next?;
        self.next = match next {
            0 => None,
            _ => Some(next - 1),
        };

        Some(PathNode {
            path: self.path,
            index: next,
            cert: self.path.get(next),
        })
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Role {
    Issuer,
    EndEntity,
}

#[cfg(all(test, feature = "alloc", any(feature = "ring", feature = "aws_lc_rs")))]
mod tests {
    use super::*;
    use crate::test_utils::{issuer_params, make_end_entity, make_issuer};
    use crate::trust_anchor::anchor_from_trusted_cert;

    #[test]
    fn eku_key_purpose_id() {
        assert!(ExtendedKeyUsage::RequiredIfPresent(EKU_SERVER_AUTH)
            .key_purpose_id_equals(EKU_SERVER_AUTH.oid_value))
    }

    #[cfg(feature = "alloc")]
    enum ChainTrustAnchor {
        NotInChain,
        InChain,
    }

    fn build_degenerate_chain(
        intermediate_count: usize,
        trust_anchor: ChainTrustAnchor,
    ) -> ControlFlow<Error, Error> {
        let ca_cert = make_issuer("Bogus Subject");
        let ca_cert_der = CertificateDer::from(ca_cert.serialize_der().unwrap());

        let mut intermediates = Vec::with_capacity(intermediate_count + 1);
        if let ChainTrustAnchor::InChain = trust_anchor {
            intermediates.push(CertificateDer::from(ca_cert_der.to_vec()));
        }

        let mut issuer = ca_cert;
        for _ in 0..intermediate_count {
            let intermediate = make_issuer("Bogus Subject");
            let intermediate_der = intermediate.serialize_der_with_signer(&issuer).unwrap();
            intermediates.push(CertificateDer::from(intermediate_der));
            issuer = intermediate;
        }

        let trust_anchor = match trust_anchor {
            ChainTrustAnchor::InChain => {
                let unused_anchor = make_issuer("Bogus Trust Anchor");
                CertificateDer::from(unused_anchor.serialize_der().unwrap())
            }
            ChainTrustAnchor::NotInChain => ca_cert_der,
        };

        let ee_der = make_end_entity(&issuer);
        let ee_cert = EndEntityCert::try_from(&ee_der).unwrap();
        verify_chain(
            &[anchor_from_trusted_cert(&trust_anchor).unwrap()],
            &intermediates,
            &ee_cert,
            None,
            None,
        )
        .map(|_| ())
        .unwrap_err()
    }

    #[test]
    fn test_too_many_signatures() {
        assert!(matches!(
            build_degenerate_chain(5, ChainTrustAnchor::NotInChain),
            ControlFlow::Break(Error::MaximumSignatureChecksExceeded)
        ));
    }

    #[test]
    fn test_too_many_path_calls() {
        assert!(matches!(
            dbg!(build_degenerate_chain(10, ChainTrustAnchor::InChain)),
            ControlFlow::Break(Error::MaximumPathBuildCallsExceeded)
        ));
    }

    fn build_linear_chain(chain_length: usize) -> Result<(), ControlFlow<Error, Error>> {
        let ca_cert = make_issuer(format!("Bogus Subject {chain_length}"));
        let ca_cert_der = CertificateDer::from(ca_cert.serialize_der().unwrap());
        let anchor = anchor_from_trusted_cert(&ca_cert_der).unwrap();
        let anchors = &[anchor.clone()];

        let mut intermediates = Vec::with_capacity(chain_length);
        let mut issuer = ca_cert;
        for i in 0..chain_length {
            let intermediate = make_issuer(format!("Bogus Subject {i}"));
            let intermediate_der = intermediate.serialize_der_with_signer(&issuer).unwrap();
            intermediates.push(CertificateDer::from(intermediate_der));
            issuer = intermediate;
        }

        let ee_der = make_end_entity(&issuer);
        let ee_cert = EndEntityCert::try_from(&ee_der).unwrap();

        let expected_chain = |path: &VerifiedPath<'_>| {
            assert_eq!(path.anchor().subject, anchor.subject);
            assert!(public_values_eq(path.end_entity().subject, ee_cert.subject));
            assert_eq!(path.intermediate_certificates().count(), chain_length);

            let intermediate_certs = intermediates
                .iter()
                .map(|der| Cert::from_der(untrusted::Input::from(der.as_ref())).unwrap())
                .collect::<Vec<_>>();

            for (cert, expected) in path
                .intermediate_certificates()
                .rev()
                .zip(intermediate_certs.iter())
            {
                assert!(public_values_eq(cert.subject, expected.subject));
                assert_eq!(cert.der(), expected.der());
            }

            for (cert, expected) in path
                .intermediate_certificates()
                .zip(intermediate_certs.iter().rev())
            {
                assert!(public_values_eq(cert.subject, expected.subject));
                assert_eq!(cert.der(), expected.der());
            }

            Ok(())
        };

        verify_chain(
            anchors,
            &intermediates,
            &ee_cert,
            Some(&expected_chain),
            None,
        )
        .map(|_| ())
    }

    #[test]
    fn longest_allowed_path() {
        assert!(build_linear_chain(1).is_ok());
        assert!(build_linear_chain(2).is_ok());
        assert!(build_linear_chain(3).is_ok());
        assert!(build_linear_chain(4).is_ok());
        assert!(build_linear_chain(5).is_ok());
        assert!(build_linear_chain(6).is_ok());
    }

    #[test]
    fn path_too_long() {
        assert!(matches!(
            build_linear_chain(7),
            Err(ControlFlow::Continue(Error::MaximumPathDepthExceeded))
        ));
    }

    #[test]
    fn name_constraint_budget() {
        // Issue a trust anchor that imposes name constraints. The constraint should match
        // the end entity certificate SAN.
        let mut ca_cert_params = issuer_params("Constrained Root");
        ca_cert_params.name_constraints = Some(rcgen::NameConstraints {
            permitted_subtrees: vec![rcgen::GeneralSubtree::DnsName(".com".into())],
            excluded_subtrees: vec![],
        });
        let ca_cert = rcgen::Certificate::from_params(ca_cert_params).unwrap();
        let ca_cert_der = CertificateDer::from(ca_cert.serialize_der().unwrap());
        let anchors = &[anchor_from_trusted_cert(&ca_cert_der).unwrap()];

        // Create a series of intermediate issuers. We'll only use one in the actual built path,
        // helping demonstrate that the name constraint budget is not expended checking certificates
        // that are not part of the path we compute.
        const NUM_INTERMEDIATES: usize = 5;
        let mut intermediates = Vec::with_capacity(NUM_INTERMEDIATES);
        for i in 0..NUM_INTERMEDIATES {
            intermediates.push(make_issuer(format!("Intermediate {i}")));
        }

        // Each intermediate should be issued by the trust anchor.
        let mut intermediates_der = Vec::with_capacity(NUM_INTERMEDIATES);
        for intermediate in &intermediates {
            intermediates_der.push(CertificateDer::from(
                intermediate.serialize_der_with_signer(&ca_cert).unwrap(),
            ));
        }

        // Create an end-entity cert that is issued by the last of the intermediates.
        let ee_der = make_end_entity(intermediates.last().unwrap());
        let ee_cert = EndEntityCert::try_from(&ee_der).unwrap();

        // We use a custom budget to make it easier to write a test, otherwise it is tricky to
        // stuff enough names/constraints into the potential chains while staying within the path
        // depth limit and the build chain call limit.
        let passing_budget = Budget {
            // One comparison against the intermediate's distinguished name.
            // One comparison against the EE's distinguished name.
            // One comparison against the EE's SAN.
            //  = 3 total comparisons.
            name_constraint_comparisons: 3,
            ..Budget::default()
        };

        // Validation should succeed with the name constraint comparison budget allocated above.
        // This shows that we're not consuming budget on unused intermediates: we didn't budget
        // enough comparisons for that to pass the overall chain building.
        let path = verify_chain(
            anchors,
            &intermediates_der,
            &ee_cert,
            None,
            Some(passing_budget),
        )
        .unwrap();
        assert_eq!(path.anchor().subject, anchors.first().unwrap().subject);

        let failing_budget = Budget {
            // See passing_budget: 2 comparisons is not sufficient.
            name_constraint_comparisons: 2,
            ..Budget::default()
        };
        // Validation should fail when the budget is smaller than the number of comparisons performed
        // on the validated path. This demonstrates we properly fail path building when too many
        // name constraint comparisons occur.
        let result = verify_chain(
            anchors,
            &intermediates_der,
            &ee_cert,
            None,
            Some(failing_budget),
        );

        assert!(matches!(
            result,
            Err(ControlFlow::Break(
                Error::MaximumNameConstraintComparisonsExceeded
            ))
        ));
    }

    #[test]
    fn test_reject_candidate_path() {
        /*
         This test builds a PKI like the following diagram depicts. We first verify
         that we can build a path EE -> B -> A -> TA. Next we supply a custom path verification
         function that rejects the B->A path, and verify that we build a path EE -> B -> C -> TA.

               ┌───────────┐
               │           │
               │     TA    │
               │           │
               └───┬───┬───┘
                   │   │
                   │   │
        ┌────────┐◄┘   └──►┌────────┐
        │        │         │        │
        │   A    │         │   C    │
        │        │         │        │
        └────┬───┘         └───┬────┘
             │                 │
             │                 │
             │   ┌─────────┐   │
             └──►│         │◄──┘
                 │    B    │
                 │         │
                 └────┬────┘
                      │
                      │
                      │
                 ┌────▼────┐
                 │         │
                 │    EE   │
                 │         │
                 └─────────┘
          */

        // Create a trust anchor, and use it to issue two distinct intermediate certificates, each
        // with a unique subject and keypair.
        let trust_anchor = make_issuer("Trust Anchor");
        let trust_anchor_der = CertificateDer::from(trust_anchor.serialize_der().unwrap());
        let trust_anchor_cert =
            Cert::from_der(untrusted::Input::from(trust_anchor_der.as_ref())).unwrap();
        let trust_anchors = &[anchor_from_trusted_cert(&trust_anchor_der).unwrap()];

        let intermediate_a = make_issuer("Intermediate A");
        let intermediate_a_der = CertificateDer::from(
            intermediate_a
                .serialize_der_with_signer(&trust_anchor)
                .unwrap(),
        );
        let intermediate_a_cert =
            Cert::from_der(untrusted::Input::from(intermediate_a_der.as_ref())).unwrap();

        let intermediate_c = make_issuer("Intermediate C");
        let intermediate_c_der = CertificateDer::from(
            intermediate_c
                .serialize_der_with_signer(&trust_anchor)
                .unwrap(),
        );
        let intermediate_c_cert =
            Cert::from_der(untrusted::Input::from(intermediate_c_der.as_ref())).unwrap();

        // Next, create an intermediate that is issued by both of the intermediates above.
        // Both should share the same subject, and key pair, but will differ in the issuer.
        let intermediate_b_key = rcgen::KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256).unwrap();
        let mut intermediate_b_params = issuer_params("Intermediate");
        intermediate_b_params.key_pair = Some(intermediate_b_key);
        let intermediate_b = rcgen::Certificate::from_params(intermediate_b_params).unwrap();

        let intermediate_b_a_der = CertificateDer::from(
            intermediate_b
                .serialize_der_with_signer(&intermediate_a)
                .unwrap(),
        );
        let intermediate_b_c_der = CertificateDer::from(
            intermediate_b
                .serialize_der_with_signer(&intermediate_c)
                .unwrap(),
        );

        let intermediates = &[
            intermediate_a_der.clone(),
            intermediate_c_der.clone(),
            intermediate_b_a_der.clone(),
            intermediate_b_c_der.clone(),
        ];

        // Create an end entity certificate signed by the keypair of the intermediates created above.
        let ee = make_end_entity(&intermediate_b);
        let ee_cert = &EndEntityCert::try_from(&ee).unwrap();

        // We should be able to create a valid path from EE to trust anchor.
        let path = verify_chain(trust_anchors, intermediates, ee_cert, None, None).unwrap();
        let path_intermediates = path.intermediate_certificates().collect::<Vec<_>>();

        // We expect that without applying any additional constraints, that the path will be
        // EE -> intermediate_b_a -> intermediate_a -> trust_anchor.
        assert_eq!(path_intermediates.len(), 2);
        assert_eq!(
            path_intermediates[0].issuer(),
            intermediate_a_cert.subject()
        );
        assert_eq!(path_intermediates[1].issuer(), trust_anchor_cert.subject());

        // Now, we'll create a function that will reject the intermediate_b_a path.
        let expected_chain = |path: &VerifiedPath<'_>| {
            for intermediate in path.intermediate_certificates() {
                // Reject any intermediates issued by intermediate A.
                if intermediate.issuer() == intermediate_a_cert.subject() {
                    return Err(Error::UnknownIssuer);
                }
            }

            Ok(())
        };

        // We should still be able to build a valid path.
        let path = verify_chain(
            trust_anchors,
            intermediates,
            ee_cert,
            Some(&expected_chain),
            None,
        )
        .unwrap();
        let path_intermediates = path.intermediate_certificates().collect::<Vec<_>>();

        // We expect that the path will now be
        // EE -> intermediate_b_c -> intermediate_c -> trust_anchor.
        assert_eq!(path_intermediates.len(), 2);
        assert_eq!(
            path_intermediates[0].issuer(),
            intermediate_c_cert.subject()
        );
        assert_eq!(path_intermediates[1].issuer(), trust_anchor_cert.subject());
    }

    fn verify_chain<'a>(
        trust_anchors: &'a [TrustAnchor<'a>],
        intermediate_certs: &'a [CertificateDer<'a>],
        ee_cert: &'a EndEntityCert<'a>,
        verify_path: Option<&dyn Fn(&VerifiedPath<'_>) -> Result<(), Error>>,
        budget: Option<Budget>,
    ) -> Result<VerifiedPath<'a>, ControlFlow<Error, Error>> {
        use core::time::Duration;

        let time = UnixTime::since_unix_epoch(Duration::from_secs(0x1fed_f00d));
        let mut path = PartialPath::new(ee_cert);
        let opts = ChainOptions {
            eku: KeyUsage::server_auth(),
            supported_sig_algs: crate::ALL_VERIFICATION_ALGS,
            trust_anchors,
            intermediate_certs,
            revocation: None,
        };

        match opts.build_chain_inner(
            &mut path,
            time,
            verify_path,
            0,
            &mut budget.unwrap_or_default(),
        ) {
            Ok(anchor) => Ok(VerifiedPath::new(ee_cert, anchor, path)),
            Err(err) => Err(err),
        }
    }
}
