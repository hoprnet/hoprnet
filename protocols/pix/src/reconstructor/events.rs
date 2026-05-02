use crate::{PixGroup, PixScalar, PixSpec, SsaIndex};

#[derive(strum::EnumTryAs, strum::EnumIs)]
pub enum ReconstructorEvent<S: PixSpec> {
    NewSsa(SsaIndex),
    SsaCommitted(SsaIndex, PixGroup<S>),
    SsaRecovered(SsaIndex, PixScalar<S>),
}

impl<S: PixSpec> std::fmt::Debug for ReconstructorEvent<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewSsa(ssi) => write!(f, "NewSsa({ssi})"),
            Self::SsaCommitted(ssi, commitment) => write!(f, "SsaCommitted({ssi}, {commitment:?})"),
            Self::SsaRecovered(ssi, _) => write!(f, "SsaRecovered({ssi}, <redacted>)"),
        }
    }
}

impl<S: PixSpec> PartialEq for ReconstructorEvent<S> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NewSsa(ssi1), Self::NewSsa(ssi2)) => ssi1 == ssi2,
            (Self::SsaCommitted(ssi1, commitment1), Self::SsaCommitted(ssi2, commitment2)) => {
                ssi1 == ssi2 && commitment1 == commitment2
            }
            (Self::SsaRecovered(ssi1, ssa1), Self::SsaRecovered(ssi2, ssa2)) => ssi1 == ssi2 && ssa1 == ssa2,
            _ => false,
        }
    }
}

impl<S: PixSpec> Eq for ReconstructorEvent<S> {}

impl<S: PixSpec> Clone for ReconstructorEvent<S> {
    fn clone(&self) -> Self {
        match self {
            Self::NewSsa(ssi) => Self::NewSsa(*ssi),
            Self::SsaCommitted(ssi, commitment) => Self::SsaCommitted(*ssi, commitment.clone()),
            Self::SsaRecovered(ssi, ssa) => Self::SsaRecovered(*ssi, ssa.clone()),
        }
    }
}
