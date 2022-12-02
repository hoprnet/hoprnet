import { Multiaddr } from '@multiformats/multiaddr'

export const PUBLIC_TCP_RFC_5780_SERVERS = [
  // see https://github.com/hoprnet/hoprnet/issues/4416
  // new Multiaddr(`/dns4/stun.hoprnet.org/tcp/3478`),
  new Multiaddr(`/dns4/stun.stunprotocol.org/tcp/3478`)
]

export const STUN_TCP_TIMEOUT = 1200
