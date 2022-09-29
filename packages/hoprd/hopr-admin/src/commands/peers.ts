import type API from '../utils/api'
import { toPaddedString } from '../utils'
import { Command, type CacheFunctions } from '../utils/command'

export default class Peers extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super({}, api, cache)
  }

  public name() {
    return 'peers'
  }

  public description() {
    return 'Lists connected and interesting HOPR nodes'
  }

  public async execute(log: (msg: string) => void, _query: string): Promise<void> {
    const peersRes = await this.api.getPeers()
    if (!peersRes.ok) return log(this.failedCommand('get peers'))
    const peers = await peersRes.json()

    const announced = peers.announced.map<[string, string]>((p) => [p.peerId, String(p.quality)])
    const connected = peers.connected.map<[string, string]>((p) => [p.peerId, String(p.quality)])

    if (announced.length === 0 && connected.length === 0) {
      return log('No peers found.')
    } else {
      return log(toPaddedString(connected))
    }
  }
}
