import PeerInfo from 'peer-info'
import PeerId from 'peer-id'
import Multiaddr from 'multiaddr'
import dns from 'dns'

type ServerMap = Map<string, PeerInfo>

const BOOTSTRAP_ADDRESS = process.env.HOPR_BOOTSTRAP_ADDRESS || '_dnsaddr.bootstrap.basodino.develop.hoprnet.org'

/** Load Bootstrap node addresses.
 *   - If a string of comma separated multiaddrs is passed, use this first
 *   - If there are ENV Variables, use them second
 *   - Otherwise look at the DNS entry for our testnet.
 */
export async function getBootstrapAddresses(addrs?: string): Promise<ServerMap> {
  let addresses: string[]
  let bootstrapServerMap = new Map<string, PeerInfo>()
  let addr: Multiaddr

  if (addrs) {
    addresses = addrs.split(',')
  } else if (process.env.HOPR_BOOTSTRAP_SERVERS !== undefined) {
    addresses = process.env.HOPR_BOOTSTRAP_SERVERS.split(',')
  } else {
    // Fall back to DNS
    let records = await dns.promises.resolveTxt(BOOTSTRAP_ADDRESS)
    addresses = records.map((r) => r[0].replace('dnsaddr=', ''))
  }

  if (addresses.length == 0) {
    throw new Error('Bootstrap Addresses could not be found')
  }

  for (let i = 0; i < addresses.length; i++) {
    addr = Multiaddr(addresses[i].trim())

    let peerInfo = bootstrapServerMap.get(addr.getPeerId())
    if (peerInfo == null) {
      peerInfo = await PeerInfo.create(PeerId.createFromB58String(addr.getPeerId()))
    }

    peerInfo.multiaddrs.add(addr)
    bootstrapServerMap.set(addr.getPeerId(), peerInfo)
  }

  return bootstrapServerMap
}
