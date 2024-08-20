#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Ord, PartialOrd)]
pub enum HoprSubProtocol {
    #[default]
    None = 0,
    StartUdpSession = 1,
    StartTcpSession = 2,
    SessionEstablished = 3,
    SessionData = 4,
    CloseSession = 5,
}
