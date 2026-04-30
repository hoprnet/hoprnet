use crate::SsaIndex;

#[derive(Debug, Clone, PartialEq, Eq, strum::EnumTryAs, strum::EnumIs)]
pub enum ReconstructorEvent {
    NewSsa(SsaIndex),
    SsaCommitted(SsaIndex),
}
