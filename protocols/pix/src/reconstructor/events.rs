use crate::{PixGroup, PixScalar, PixSpec, SsaIndex};

#[derive(Debug, Clone, PartialEq, Eq, strum::EnumTryAs, strum::EnumIs)]
pub enum ReconstructorEvent<S: PixSpec + 'static> {
    NewSsa(SsaIndex),
    SsaCommitted(SsaIndex, PixGroup<S>),
    SsaRecovered(SsaIndex, PixScalar<S>),
}
