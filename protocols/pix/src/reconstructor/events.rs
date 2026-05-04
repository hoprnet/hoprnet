use crate::{PixGroupRepr, PixScalar, PixSpec, types::SsaId};

#[derive(strum::EnumTryAs, strum::EnumIs)]
pub enum ReconstructorEvent<S: PixSpec> {
    /// Emitted when a new SSA is encountered for the first time.
    NewSsa(SsaId<S>),
    /// Emitted when the commitment to an SSA is known and can be checked for deposit already.
    SsaCommitmentKnown(SsaId<S>, PixGroupRepr<S>),
    /// Emitted when a new SSA is completely committed to by the client and can therefore
    /// be used to for RP traffic.
    SsaFullyCommitted(SsaId<S>, PixGroupRepr<S>),
    /// Emitted whenever the private scalar of a SSA is fully reconstructed.
    SsaRecovered(SsaId<S>, PixScalar<S>),
}

impl<S: PixSpec> std::fmt::Debug for ReconstructorEvent<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewSsa(id) => write!(f, "NewSsa({id})"),
            Self::SsaFullyCommitted(id, commitment) => write!(f, "SsaCommitted({id}, {commitment:?})"),
            Self::SsaRecovered(id, _) => write!(f, "SsaRecovered({id}, <redacted>)"),
            Self::SsaCommitmentKnown(id, addr) => write!(f, "SsaCommitmentKnown({id}, {addr:?})"),
        }
    }
}

impl<S: PixSpec> PartialEq for ReconstructorEvent<S> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NewSsa(id1), Self::NewSsa(id2)) => id1 == id2,
            (Self::SsaFullyCommitted(id1, commitment1), Self::SsaFullyCommitted(id2, commitment2)) => {
                id1 == id2 && commitment1 == commitment2
            }
            (Self::SsaRecovered(id1, ssa1), Self::SsaRecovered(id2, ssa2)) => id1 == id2 && ssa1 == ssa2,
            (Self::SsaCommitmentKnown(id1, addr1), Self::SsaCommitmentKnown(id2, addr2)) => {
                id1 == id2 && addr1 == addr2
            }
            _ => false,
        }
    }
}

impl<S: PixSpec> Eq for ReconstructorEvent<S> {}

impl<S: PixSpec> Clone for ReconstructorEvent<S> {
    fn clone(&self) -> Self {
        match self {
            Self::NewSsa(id) => Self::NewSsa(*id),
            Self::SsaFullyCommitted(id, commitment) => Self::SsaFullyCommitted(*id, *commitment),
            Self::SsaRecovered(id, ssa) => Self::SsaRecovered(*id, *ssa),
            Self::SsaCommitmentKnown(id, addr) => Self::SsaCommitmentKnown(*id, *addr),
        }
    }
}
