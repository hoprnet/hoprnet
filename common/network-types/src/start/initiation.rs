use crate::start::errors::StartError;
use crate::types::RoutingOptions;
use std::iter::Extend;

pub type StartChallenge = u64;

#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::EnumDiscriminants, strum::EnumTryAs)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::EnumDiscriminants, strum::EnumTryAs)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[strum_discriminants(derive(strum::FromRepr), repr(u8))]
pub enum StartSessionTarget {
    /// Target is running over UDP with the given address and port.
    /// Currently, only IPv4 is supported.
    #[cfg_attr(feature = "serde", serde_as(as = "DisplayFromStr"))]
    UdpStream(std::net::SocketAddrV4),
    /// Target is running over TCP with the given address and port.
    /// Currently, only IPv4 is supported.
    #[cfg_attr(feature = "serde", serde_as(as = "DisplayFromStr"))]
    TcpStream(std::net::SocketAddrV4),
    /// Target is a service directly at the exit node with a given service ID.
    ExitNode(u32),
}

impl<'a> TryFrom<&'a [u8]> for StartSessionTarget {
    type Error = StartError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match value
            .get(0)
            .ok_or(StartError::InvalidMessageLength)
            .and_then(|v| StartSessionTargetDiscriminants::from_repr(*v).ok_or(StartError::ParseError))?
        {
            d @ StartSessionTargetDiscriminants::UdpStream | StartSessionTargetDiscriminants::TcpStream => {
                // IPv6 currently not supported
                if value.len() == 1 + 4 /* ipv4 addr len */ + 2
                /* port len */
                {
                    let (addr, port) = value[1..].split_at(4);
                    let sa = std::net::SocketAddrV4::new(
                        std::net::Ipv4Addr::from(addr.try_into().map_err(|_| StartError::ParseError)?),
                        u16::from_be_bytes(port.try_into().map_err(|_| StartError::ParseError)?),
                    );

                    Ok(match d {
                        StartSessionTargetDiscriminants::UdpStream => StartSessionTarget::UdpStream(sa),
                        StartSessionTargetDiscriminants::TcpStream => StartSessionTarget::TcpStream(sa),
                        _ => unreachable!(),
                    })
                } else {
                    Err(StartError::InvalidMessageLength)
                }
            }
            StartSessionTargetDiscriminants::ExitNode => {
                if value.len() == 1 + size_of::<u32>() {
                    let service_id = u32::from_be_bytes(value[1..].try_into().map_err(|_| StartError::ParseError)?);
                    Ok(StartSessionTarget::ExitNode(service_id))
                } else {
                    Err(StartError::InvalidMessageLength)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StartInitiation {
    pub challenge: StartChallenge,
    pub target: StartSessionTarget,
    pub back_routing: RoutingOptions, // TODO: removed in 3.0 when return path is introduced
}

impl<'a> TryFrom<&'a [u8]> for StartInitiation {
    type Error = StartError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StartEstablished<T> {
    response: StartChallenge,
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[strum_discriminants(derive(strum::FromRepr), repr(u8))]
pub enum StartProtocol<T> {
    StartSession(StartInitiation),
    SessionEstablished(StartEstablished<T>),
    SessionError(StartInternalError),
    CloseSession(T),
}

impl<T> StartProtocol<T>
where
    T: for<'a> TryFrom<&'a [u8]>,
{
    pub fn from_tag_and_data(tag: u8, data: &[u8]) -> super::errors::Result<Self> {
        match StartProtocolDiscriminants::from_repr(tag).ok_or(StartError::ParseError)? {
            StartProtocolDiscriminants::StartSession => {
                StartInitiation::try_from(data).map(StartProtocol::StartSession)
            }
            StartProtocolDiscriminants::SessionEstablished => {
                StartEstablished::try_from(data).map(StartProtocol::SessionEstablished)
            }
            StartProtocolDiscriminants::SessionError => {
                StartInternalError::try_from(data).map(StartProtocol::SessionError)
            }
            StartProtocolDiscriminants::CloseSession => T::try_from(data)
                .map(StartProtocol::CloseSession)
                .map_err(|_| StartError::ParseError),
        }
    }

    pub fn encode(self) -> Box<[u8]> {
        let mut out = Vec::with_capacity(128);
        let disc = StartProtocolDiscriminants::from(&self) as u8;
        out.push(disc);
        match self {
            StartProtocol::StartSession(init) => out.extend(init.encode()),
            StartProtocol::SessionEstablished(est) => out.extend(est.encode()),
            StartProtocol::SessionError(err) => out.extend(err.encode()),
            StartProtocol::CloseSession(id) => {}
        }
    }
}
