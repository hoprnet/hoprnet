use crate::start::errors::StartError;

pub type StartChallenge = u64;

#[derive(Debug, Copy, Clone, PartialEq, strum::EnumDiscriminants, strum::EnumTryAs, Eq)]
#[strum_discriminants(derive(strum::FromRepr), repr(u8))]
pub enum StartInternalError {
    SessionDenied(StartChallenge),
}

impl<'a> TryFrom<&'a [u8]> for StartInternalError {
    type Error = StartError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match value
            .get(0)
            .ok_or(StartError::InvalidMessageLength)
            .and_then(|v| StartInternalErrorDiscriminants::from_repr(*v).ok_or(StartError::ParseError))?
        {
            StartInternalErrorDiscriminants::SessionDenied => {
                let challenge = StartChallenge::from_be_bytes(
                    value[1..size_of::<StartChallenge>()]
                        .try_into()
                        .map_err(|_| StartError::ParseError)?,
                );
                Ok(StartInternalError::SessionDenied(challenge))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartInitiation {
    pub challenge: StartChallenge,
    pub target: std::net::SocketAddr,
}

impl<'a> TryFrom<&'a [u8]> for StartInitiation {
    type Error = StartError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartEstablished<T> {
    challenge: StartChallenge,
    session_id: T,
}

impl<'a, T> TryFrom<&'a [u8]> for StartEstablished<T>
where
    T: TryFrom<&'a [u8]>,
{
    type Error = StartError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants, strum::EnumTryAs)]
#[strum_discriminants(derive(strum::FromRepr), repr(u8))]
pub enum StartProtocol<'a, T> {
    None(&'a [u8]),
    StartUdpSession(StartInitiation),
    StartTcpSession(StartInitiation),
    SessionEstablished(StartEstablished<T>),
    SessionError(StartInternalError),
    SessionData(&'a [u8]),
    CloseSession(T),
}

impl<'a, T> TryFrom<&'a [u8]> for StartProtocol<'a, T>
where
    T: TryFrom<&'a [u8]>,
{
    type Error = StartError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match value
            .get(0)
            .ok_or(StartError::InvalidMessageLength)
            .and_then(|v| StartProtocolDiscriminants::from_repr(*v).ok_or(StartError::ParseError))?
        {
            StartProtocolDiscriminants::None => Ok(StartProtocol::None(&value[1..])),
            StartProtocolDiscriminants::StartUdpSession => {
                StartInitiation::try_from(&value[1..]).map(StartProtocol::StartUdpSession)
            }
            StartProtocolDiscriminants::StartTcpSession => {
                StartInitiation::try_from(&value[1..]).map(StartProtocol::StartTcpSession)
            }
            StartProtocolDiscriminants::SessionEstablished => {
                StartEstablished::try_from(&value[1..]).map(StartProtocol::SessionEstablished)
            }
            StartProtocolDiscriminants::SessionError => {
                StartInternalError::try_from(&value[1..]).map(StartProtocol::SessionError)
            }
            StartProtocolDiscriminants::SessionData => Ok(StartProtocol::SessionData(&value[1..])),
            StartProtocolDiscriminants::CloseSession => T::try_from(&value[1..])
                .map(StartProtocol::CloseSession)
                .map_err(|_| StartError::ParseError),
        }
    }
}
