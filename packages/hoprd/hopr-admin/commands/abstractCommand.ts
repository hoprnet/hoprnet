import { styleValue } from './utils'
import HoprFetcher from '../fetch'
import PeerId from 'peer-id'

// REPL Command
export abstract class AbstractCommand {
  public hidden = false
  public hoprFetcher: HoprFetcher

  protected constructor(fetcher: HoprFetcher) {
    this.hoprFetcher = fetcher
  }

  // The command, for example 'ping' or 'foo'
  abstract name(): string

  // A help string describing the command
  abstract help(): string

  // Run the command with optional argument
  abstract execute(log: (string) => void, query: string): Promise<void>

  protected usage(parameters: string[]): string {
    return `usage: ${parameters.map((x) => `<${x}>`).join(' ')}`
  }

  // returns [error, ...params]
  protected _assertUsage(query: string, parameters: string[], test?: RegExp): string[] {
    const usage = styleValue(this.usage(parameters), 'failure')

    if (!query && parameters.length > 0) {
      return [usage]
    }

    let match: string[] = []
    // uses RegExp
    if (test) {
      match = Array.from(test.exec(query) ?? [])
      // remove the first match as it is the command name
      match.shift()
    }
    // simply split by space
    else {
      match = query.split(' ')
    }

    if (match.length !== parameters.length) {
      return [usage]
    }

    //@ts-ignore : The first element is a string|undefined, but typing this is a nightmare
    return [undefined].concat(parameters.map((x, i) => match[i]))
  }

  /**
   * Takes a string, and checks whether it's an alias or a valid peerId,
   * then it generates a PeerId instance and returns it.
   *
   * @param peerIdString query that contains the peerId
   * @returns a 'PeerId' instance
   */
  public async checkPeerIdInput(peerIdString: string): Promise<PeerId> {
    const aliases: string[] = await this.hoprFetcher.getAliases().then(res => Object.values(res))

    try {
      if (typeof aliases !== 'undefined' && aliases && aliases.includes(peerIdString)) {
        return PeerId.createFromB58String(peerIdString)
      }

      return PeerId.createFromB58String(peerIdString)
    } catch (err) {
      throw Error(`Invalid peerId. ${err.message}`)
    }
  }

}
