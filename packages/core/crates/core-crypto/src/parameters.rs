/// Length of a secret key
pub const SECRET_KEY_LENGTH: usize = 32;

/// Length of a packet tag
pub const PACKET_TAG_LENGTH: usize = 16;

/// Length of the ping challenge and pong response in bytes
pub const PING_PONG_NONCE_SIZE: usize = 16;

/// Length of message authentication code (used in packet, etc.)
pub const MAC_LENGTH: usize = 32;