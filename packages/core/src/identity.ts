import type { HoprOptions } from '.'
import PeerId from 'peer-id'
import { Multiaddr } from 'multiaddr'

const DEFAULT_PORT = 9091

/**
 * Assemble the addresses that we are using
 */
export function getAddrs(id: PeerId, options: HoprOptions): Multiaddr[] {
  const addrs = []

  if (options.hosts === undefined || (options.hosts.ip4 === undefined && options.hosts.ip6 === undefined)) {
    addrs.push(new Multiaddr(`/ip4/0.0.0.0/tcp/${DEFAULT_PORT}`))
  }

  if (options.hosts !== undefined) {
    if (options.hosts.ip4 === undefined && options.hosts.ip6 === undefined) {
      throw Error(`Unable to detect to which interface we should listen`)
    }

    if (options.hosts.ip4 !== undefined) {
      addrs.push(new Multiaddr(`/ip4/${options.hosts.ip4.ip}/tcp/${options.hosts.ip4.port}`))
    }

    if (options.hosts.ip6 !== undefined) {
      addrs.push(new Multiaddr(`/ip6/${options.hosts.ip6.ip}/tcp/${options.hosts.ip6.port}`))
    }
  }

  return addrs.map((addr: Multiaddr) => addr.encapsulate(`/p2p/${id.toB58String()}`))
}
