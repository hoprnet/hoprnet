import type { Multiaddr } from 'multiaddr'
import type { Network } from './utils/constants'
import {
  getPrivateAddresses,
  isPrivateAddress,
  checkNetworks,
  isLinkLocaleAddress,
  parseAddress,
  u8aAddrToString
} from './utils'
import type { ValidAddress } from './utils'

import type { NetworkInterfaceInfo } from 'os'
import type PeerId from 'peer-id'
import { u8aEquals } from '@hoprnet/hopr-utils'
import Debug from 'debug'
import { green } from 'chalk'
import assert from 'assert'

const log = Debug('hopr-connect:filter')

const INVALID_PORTS = [0]

export class Filter {
  private announcedAddrs?: ValidAddress[]
  private listeningFamilies?: NetworkInterfaceInfo['family'][]

  protected myPrivateNetworks: Network[]

  constructor(private peerId: PeerId) {
    this.myPrivateNetworks = getPrivateAddresses()
  }

  /**
   * Used to check whether addresses have already been attached
   */
  get addrsSet(): boolean {
    return this.announcedAddrs != undefined && this.listeningFamilies != undefined
  }

  /**
   * Used to attach addresses once libp2p is initialized and
   * sockets are bound to network interfaces
   * @param announcedAddrs Addresses that are announced to other nodes
   * @param listeningAddrs Addresses to which we are listening
   */
  setAddrs(announcedAddrs: Multiaddr[], listeningAddrs: Multiaddr[]): void {
    log(`announcedAddrs:`)
    announcedAddrs.forEach((ma: Multiaddr) => log(` ${green(ma.toString())}`))
    log(`listeningAddrs:`)
    listeningAddrs.forEach((ma: Multiaddr) => log(` ${green(ma.toString())}`))

    this.announcedAddrs = []
    for (const announcedAddr of announcedAddrs) {
      const parsed = parseAddress(announcedAddr)

      if (parsed.valid) {
        this.announcedAddrs.push(parsed.address)
      }
    }

    this.listeningFamilies = []
    for (const listenAddr of listeningAddrs) {
      const parsed = parseAddress(listenAddr)

      assert(parsed.valid)
      switch (parsed.address.type) {
        case 'IPv4':
        case 'IPv6':
          if (!this.listeningFamilies.includes(parsed.address.type)) {
            this.listeningFamilies.push(parsed.address.type)
          }
        case 'p2p':
          continue
      }
    }
  }

  public filter(ma: Multiaddr) {
    if (this.addrsSet) {
      return this.filterDial(ma)
    } else {
      return this.filterListening(ma)
    }
  }

  /**
   * Check whether we can listen to the given Multiaddr
   * @param ma Multiaddr to check
   * @returns true if address is usable
   */
  public filterListening(ma: Multiaddr): boolean {
    const parsed = parseAddress(ma)

    if (!parsed.valid || !['IPv4', 'IPv6'].includes(parsed.address.type)) {
      log(`Can only listen to valid IP addresses. Given addr: ${ma.toString()}`)
      return false
    }

    if (parsed.address.node != undefined && !u8aEquals(parsed.address.node, this.peerId.marshalPubKey())) {
      log(`Cannot listen to multiaddrs with other peerId than our own. Given addr: ${ma.toString()}`)
      return false
    }

    return true
  }

  /**
   * Check whether it makes sense to dial the given address
   * @param ma Multiaddress to check
   * @returns true if considered dialable
   */
  public filterDial(ma: Multiaddr): boolean {
    const parsed = parseAddress(ma)

    if (!parsed.valid) {
      return false
    }

    if (parsed.address.type === 'p2p') {
      const address = parsed.address

      if (u8aEquals(address.node, this.peerId.marshalPubKey())) {
        log(`Prevented self-dial using circuit addr. Used addr: ${ma.toString()}`)
        return false
      }

      if (u8aEquals(address.relayer, this.peerId.marshalPubKey())) {
        log(`Prevented dial using self as relay node. Used addr: ${ma.toString()}`)
        return false
      }

      return true
    }

    const address = parsed.address

    if (address.node != undefined && u8aEquals(address.node, this.peerId.marshalPubKey())) {
      log(`Prevented self-dial. Used addr: ${ma.toString()}`)
      return false
    }

    assert(this.announcedAddrs != undefined && this.listeningFamilies != undefined)

    if (!this.listeningFamilies.includes(address.type)) {
      // Prevent dialing IPv6 addresses when only listening to IPv4 and vice versa
      log(`Tried to dial ${parsed.address.type} address but listening to ${this.listeningFamilies.join(', ')}`)
      return false
    }

    if (INVALID_PORTS.includes(address.port)) {
      log(`Tried to dial invalid port ${address.port}`)
      return false
    }

    // Allow multiple nodes on same host - independent of address type
    for (const announcedAddr of this.announcedAddrs) {
      if (announcedAddr.type === 'p2p') {
        continue
      }

      if (announcedAddr.type === address.type && u8aEquals(announcedAddr.address, address.address)) {
        // Always allow dials to own address whenever port is different
        // and block if port is identical
        if (address.port == announcedAddr.port) {
          log(
            `Prevented dialing ${u8aAddrToString(address.address, address.type)}:${
              address.port
            } because self listening on ${u8aAddrToString(announcedAddr.address, announcedAddr.type)}:${
              announcedAddr.port
            }`
          )
        }
        return address.port != announcedAddr.port
      }
    }

    if (isLinkLocaleAddress(address.address, address.type)) {
      log(`Cannot dial link-locale addresses. Used address ${u8aAddrToString(address.address, address.type)}`)
      return false
    }

    if (isPrivateAddress(address.address, address.type)) {
      if (!checkNetworks(this.myPrivateNetworks, address.address, address.type)) {
        log(
          `Prevented dialing private address ${u8aAddrToString(address.address, address.type)}:${
            address.port
          } because not in our network(s): ${this.myPrivateNetworks
            .map((network) => `${u8aAddrToString(network.networkPrefix, network.family)}`)
            .join(', ')}`
        )
        return false
      }
    }

    return true
  }
}
