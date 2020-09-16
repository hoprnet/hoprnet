// This should filter IP4's from private networks, as defined by RFC1918
export const PRIVATE_NETS = /(^127\.)|(^10\.)|(^172\.1[6-9]\.)|(^172\.2[0-9]\.)|(^172\.3[0-1]\.)|(^192\.168\.)/
