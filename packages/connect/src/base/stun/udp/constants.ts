import { Multiaddr } from '@multiformats/multiaddr'

export const STUN_UDP_TIMEOUT = 700

// To be extended
export const PUBLIC_UDP_RFC_5780_SERVERS = [
  // see https://github.com/hoprnet/hoprnet/issues/4416
  // new Multiaddr(`/dns4/stun.hoprnet.org/tcp/3478`),
  new Multiaddr(`/dns4/stun.bluesip.net/udp/3478`),
  new Multiaddr(`/dns4/stun.stunprotocol.org/udp/3478`)
]

// Only used to determine the external address of the bootstrap server
export const PUBLIC_UDP_STUN_SERVERS = [
  new Multiaddr(`/dns4/stun.l.google.com/udp/19302`),
  new Multiaddr(`/dns4/stun1.l.google.com/udp/19302`),
  new Multiaddr(`/dns4/stun2.l.google.com/udp/19302`),
  new Multiaddr(`/dns4/stun3.l.google.com/udp/19302`),
  new Multiaddr(`/dns4/stun4.l.google.com/udp/19302`),
  new Multiaddr(`/dns4/stun.sipgate.net/udp/3478`),
  new Multiaddr(`/dns4/stun.callwithus.com/udp/3478`)
]
