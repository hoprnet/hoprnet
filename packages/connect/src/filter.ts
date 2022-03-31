import type { Multiaddr } from 'multiaddr'
import type { ValidAddress } from './utils'
import type PeerId from 'peer-id'

import {
  u8aEquals,
  checkNetworks,
  isPrivateAddress,
  isLinkLocaleAddress,
  isReservedAddress,
  u8aAddrToString,
  getPrivateAddresses,
  isLocalhost,
  u8aAddressToCIDR,
  type Network
} from '@hoprnet/hopr-utils'
import { AddressType, parseAddress } from './utils'

import Debug from 'debug'
import { HoprConnectOptions } from './types'

const log = Debug('hopr-connect:filter')

const INVALID_PORTS = [0]

export class Filter {
  private announcedAddrs?: ValidAddress[]
  private listeningFamilies?: (AddressType.IPv4 | AddressType.IPv6)[]
  private myPublicKey: Uint8Array

  protected myPrivateNetworks: Network[]

  constructor(peerId: PeerId, private opts: HoprConnectOptions) {
    this.myPrivateNetworks = getPrivateAddresses()
    this.myPublicKey = peerId.marshalPubKey()
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
    this.announcedAddrs = []
    for (const announcedAddr of announcedAddrs) {
      const parsed = parseAddress(announcedAddr)

      if (!parsed.valid) {
        continue
      }

      this.announcedAddrs.push(parsed.address)
    }

    this.listeningFamilies = []
    for (const listenAddr of listeningAddrs) {
      const parsed = parseAddress(listenAddr)

      if (!parsed.valid) {
        continue
      }

      switch (parsed.address.type) {
        case AddressType.IPv4:
        case AddressType.IPv6:
          if (!this.listeningFamilies.includes(parsed.address.type)) {
            this.listeningFamilies.push(parsed.address.type)
          }
          break
        case AddressType.P2P:
          continue
      }
    }
  }

  public filter(ma: Multiaddr): boolean {
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
  private filterListening(ma: Multiaddr): boolean {
    const parsed = parseAddress(ma)

    if (!parsed.valid) {
      return false
    }

    switch (parsed.address.type) {
      case AddressType.IPv4:
      case AddressType.IPv6:
        if (parsed.address.node != undefined && !u8aEquals(parsed.address.node, this.myPublicKey)) {
          log(`Cannot listen to multiaddrs with other peerId than our own. Given addr: ${ma.toString()}`)
          return false
        }

        return true
      case AddressType.P2P:
        log(`Can only listen to IP addresses: Given addr: ${ma.toString()}`)
        return false
    }
  }

  /**
   * Check whether it makes sense to dial the given address
   * @param ma Multiaddress to check
   * @returns true if considered dialable
   */
  private filterDial(ma: Multiaddr): boolean {
    const parsed = parseAddress(ma)

    if (!parsed.valid) {
      return false
    }

    switch (parsed.address.type) {
      case AddressType.P2P:
        if (u8aEquals(parsed.address.node, this.myPublicKey)) {
          log(`Prevented self-dial using circuit addr. Used addr: ${ma.toString()}`)
          return false
        }

        if (u8aEquals(parsed.address.relayer, this.myPublicKey)) {
          log(`Prevented dial using self as relay node. Used addr: ${ma.toString()}`)
          return false
        }

        break
      case AddressType.IPv4:
      case AddressType.IPv6:
        if (parsed.address.node != undefined && u8aEquals(parsed.address.node, this.myPublicKey)) {
          log(`Prevented self-dial. Used addr: ${ma.toString()}`)
          return false
        }

        if (!this.listeningFamilies!.includes(parsed.address.type)) {
          // Prevent dialing IPv6 addresses when only listening to IPv4 and vice versa
          log(`Tried to dial ${parsed.address.type} address but listening to ${this.listeningFamilies!.join(', ')}`)
          return false
        }

        if (INVALID_PORTS.includes(parsed.address.port)) {
          log(`Tried to dial invalid port ${parsed.address.port}`)
          return false
        }

        // Prevent dialing any link-locale addresses or reserved addresses
        if (
          isLinkLocaleAddress(parsed.address.address, parsed.address.type) ||
          isReservedAddress(parsed.address.address, parsed.address.type)
        ) {
          return false
        }

        if (isLocalhost(parsed.address.address, parsed.address.type)) {
          // If localhost connections are explicitly allowed, do not dial them
          if (!this.opts.allowLocalConnections) {
            // Do not pollute logs by rejecting localhost connections attempts
            return false
          }

          // Allow to dial localhost only if the port is different from all of those we're listening on
          if (
            this.announcedAddrs!.some(
              (announced: ValidAddress) =>
                announced.type !== AddressType.P2P &&
                isLocalhost(announced.address, announced.type) &&
                announced.type === parsed.address.type &&
                announced.port == parsed.address.port
            )
          ) {
            // Do not log anything to prevent too much log pollution
            return false
          }
        }

        if (isPrivateAddress(parsed.address.address, parsed.address.type)) {
          // If private address connections are explicitly allowed, do not dial them
          if (!this.opts.allowPrivateConnections) {
            // Do not pollute logs by rejecting private address connections attempts
            return false
          }

          // If different private network, there is most likely no chance to establish a connection
          if (!checkNetworks(this.myPrivateNetworks, parsed.address.address, parsed.address.type)) {
            log(
              `Prevented dialing private address ${u8aAddrToString(parsed.address.address, parsed.address.type)}:${
                parsed.address.port
              } because not in our network(s): ${this.myPrivateNetworks
                .map((network) => `${u8aAddressToCIDR(network.networkPrefix, network.subnet, network.family)}`)
                .join(', ')}`
            )
            return false
          }
        }

        // Allow multiple nodes on same host - independent of address type
        for (const announcedAddr of this.announcedAddrs!) {
          switch (announcedAddr.type) {
            case AddressType.P2P:
              continue
            case AddressType.IPv4:
            case AddressType.IPv6:
              if (u8aEquals(announcedAddr.address, parsed.address.address)) {
                // Always allow dials to own address whenever port is different
                // and block if port is identical
                if (parsed.address.port == announcedAddr.port) {
                  log(
                    `Prevented dialing ${u8aAddrToString(parsed.address.address, parsed.address.type)}:${
                      parsed.address.port
                    } because self listening on ${u8aAddrToString(announcedAddr.address, announcedAddr.type)}:${
                      announcedAddr.port
                    }`
                  )
                  return false
                }
              }
          }
        }
    }

    return true
  }
}
