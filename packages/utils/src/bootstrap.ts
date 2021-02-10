import Multiaddr from 'multiaddr'
import dns from 'dns'
import semver from 'semver'

const FULL_VERSION = require('../package.json').version
const cleanVersion = (version) =>
  `${semver.major(version)}.${semver.minor(version)}.${semver.patch(version)}`

const BOOTSTRAP_ADDRESS = process.env.HOPR_BOOTSTRAP_ADDRESS || `${cleanVersion(FULL_VERSION)}-bootstrap.hoprnet.link`

/** Load Bootstrap node addresses.
 *   - If a string of comma separated multiaddrs is passed, use this first
 *   - If there are ENV Variables, use them second
 *   - Otherwise look at the DNS entry for our testnet.
 */
export async function getBootstrapAddresses(addrs?: string): Promise<Multiaddr[]> {
  let addresses: string[]

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
