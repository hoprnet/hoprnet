import PeerId from 'peer-id'
import Multiaddr from 'multiaddr'
import dns from 'dns'

const BOOTSTRAP_ADDRESS = process.env.HOPR_BOOTSTRAP_ADDRESS || '_dnsaddr.bootstrap.zurich.telemetry.develop.hoprnet.org'

/** Load Bootstrap node addresses.
 *   - If a string of comma separated multiaddrs is passed, use this first
 *   - If there are ENV Variables, use them second
 *   - Otherwise look at the DNS entry for our testnet.
 */
export async function getBootstrapAddresses(addrs?: string): Promise<Multiaddr[]> {
  let addresses: string[]
  let servers: Multiaddr[] = []

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

  return addresses.map((a: string) => Multiaddr(a.trim()))
}
