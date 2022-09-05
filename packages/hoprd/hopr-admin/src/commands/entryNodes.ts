import type API from '../utils/api'
import { Command, type CacheFunctions } from '../utils/command'

const COLLAPSED_PEER_ID_LENGTH = 5
const ELIGIBLE_PREFIX = 'Eligible'
const MULTIADDRS_PREFIX = 'Multiaddrs'
const PREFIX = `${'Id'.padEnd(COLLAPSED_PEER_ID_LENGTH, ' ')}  ${ELIGIBLE_PREFIX}  ${MULTIADDRS_PREFIX}`

export default class EntryNodes extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super({}, api, cache, true)
  }

  public name() {
    return 'entryNodes'
  }

  public description() {
    return 'Lists announced entry nodes and their eligibility status'
  }

  public async execute(log: (msg: string) => void, _query: string): Promise<void> {
    const entryNodesRes = await this.api.getEntryNodes()
    if (!entryNodesRes.ok) {
      return log(this.failedCommand('get entryNodes'))
    }

    const peers = await entryNodesRes.json()

    let out = `${PREFIX}\n`
    for (const [id, entry] of Object.entries(peers)) {
      if (out.length > PREFIX.length + 1) {
        out += '\n'
      }
      out += `${id}  `
      out += `${String(entry.isEligible).padEnd(ELIGIBLE_PREFIX.length)}  `
      out += entry.multiaddrs.join(',')
    }

    if (out.length === PREFIX.length + 1) {
      return log('No entry nodes known.')
    } else {
      return log(out)
    }
  }
}
