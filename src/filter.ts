import Multiaddr from 'multiaddr'
import type { Network } from './utils/constants'
import { getLocalAddresses, isPrivateAddress, checkNetworks, isLinkLocaleAddress } from './utils'
import { CODE_IP4, CODE_IP6, CODE_P2P, CODE_CIRCUIT, CODE_TCP } from './constants'
import Multihash from 'multihashes'
import { NetworkInterfaceInfo } from 'os'
import PeerId from 'peer-id'
import { u8aEquals, u8aToNumber } from '@hoprnet/hopr-utils'
import Debug from 'debug'
import { green } from 'chalk'

const log = Debug('hopr-connect:filter')

const INVALID_PORTS = [0]

function checkCircuitAddress(maTuples: [code: number, addr: Uint8Array][], peerId: PeerId): boolean {
  if (
    maTuples.length != 3 ||
    maTuples[0][0] != CODE_P2P ||
    maTuples[1][0] != CODE_CIRCUIT ||
    maTuples[2][0] != CODE_P2P
  ) {
    return false
  }

  // first address and second address WITHOUT length prefix
  const [firstAddress, secondAddress] = [maTuples[0][1].slice(1), maTuples[2][1].slice(1)]

  try {
    // Try to decode first node address
    Multihash.validate(firstAddress) // throws if invalid
  } catch (err) {
    // Could not decode address
    console.log(`first address not valid`, err)
    return false
  }

  if (u8aEquals(firstAddress, peerId.toBytes())) {
    // Cannot use ourself to relay traffic
    return false
  }

  try {
    // Try to decode second node address
    Multihash.validate(secondAddress) // throws if invalid
  } catch (err) {
    return false
  }

  if (u8aEquals(firstAddress, secondAddress)) {
    // Relay and recipient must be different
    return false
  }

  return true
}

export class Filter {
  private announcedAddrs?: Multiaddr[]
  private listenFamilies?: number[]

  private myLocalAddresses: Network[]

  constructor(private peerId: PeerId) {
    this.myLocalAddresses = getLocalAddresses()
  }

  /**
   * Used to check whether addresses have already been attached
   */
  get addrsSet(): boolean {
    return this.announcedAddrs == undefined || this.listenFamilies == undefined
  }

  /**
   * THIS METHOD IS USED FOR TESTING
   * @dev Used to set falsy local network
   * @param mAddrs new local addresses
   */
  _setLocalAddressesForTesting(networks: Network[]): void {
    this.myLocalAddresses = networks
  }

  /**
   * Used to attach addresses once libp2p is initialized and
   * sockets are bound to network interfaces
   * @param announcedAddrs Addresses that are announced to other nodes
   * @param listeningAddrs Addresses to which we are listening
   */
  setAddrs(announcedAddrs: Multiaddr[], listeningAddrs: Multiaddr[]): void {
    log(
      `announcedAddrs`,
      announcedAddrs.map((ma: Multiaddr) => green(ma.toString())).join(','),
      `listeningAddrs`,
      listeningAddrs.map((ma: Multiaddr) => green(ma.toString())).join(',')
    )
    this.announcedAddrs = announcedAddrs
    this.listenFamilies = []

    for (const listenAddr of listeningAddrs) {
      const listenTuples = listenAddr.tuples()

      switch (listenTuples[0][0]) {
        case CODE_IP4:
          if (!this.listenFamilies.includes(CODE_IP4)) {
            this.listenFamilies.push(CODE_IP4)
          }
          break
        case CODE_IP6:
          if (!this.listenFamilies.includes(CODE_IP6)) {
            this.listenFamilies.push(CODE_IP6)
          }
          break
      }
    }
  }

  /**
   * Used to check whether we can listen to addresses and
   * if we can dial these addresses
   * @param ma Address to check
   */
  public filter(ma: Multiaddr): boolean {
    const tuples = ma.tuples()
    let family: NetworkInterfaceInfo['family']

    switch (tuples[0][0]) {
      case CODE_IP4:
        family = 'IPv4'
        break
      case CODE_IP6:
        family = 'IPv6'
        break
      case CODE_P2P:
        return checkCircuitAddress(tuples, this.peerId)
      default:
        return false
    }

    if (tuples[1][0] != CODE_TCP) {
      // We are not listening to anything else than TCP
      return false
    }

    const [ipFamily, ipAddr, tcpPort] = [...tuples[0], tuples[1][1]]

    if (isLinkLocaleAddress(ipAddr, family)) {
      // Cannot bind or listen to link-locale addresses
      return false
    }

    if (this.listenFamilies == undefined || this.announcedAddrs == undefined) {
      // Libp2p has not been initialized
      return true
    }

    // Only check for invalid Ports if libp2p is initialized
    if (INVALID_PORTS.includes(u8aToNumber(tcpPort) as number)) {
      return false
    }

    if (!this.listenFamilies.includes(ipFamily)) {
      // It seems that we are not listening to this address family
      return false
    }

    for (const announcedAddr of this.announcedAddrs) {
      if (ma.decapsulateCode(CODE_P2P).equals(announcedAddr.decapsulateCode(CODE_P2P))) {
        // Address is our own address, reject
        return false
      }
    }

    if (isPrivateAddress(ipAddr, family)) {
      if (!checkNetworks(this.myLocalAddresses, ipAddr, family)) {
        return false
      }
    }

    return true
  }
}
