import type PeerId from 'peer-id'
import { styleValue } from './utils/index.js'

export type GlobalState = {
  aliases: Map<string, PeerId>
  includeRecipient: boolean
}

// REPL Command
export abstract class AbstractCommand {
  public hidden = false

  // The command, for example 'ping' or 'foo'
  abstract name(): string

  // A help string describing the command
  abstract help(): string

  // Run the command with optional argument
  abstract execute(log: (string) => void, query: string, state: GlobalState): Promise<void>

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
}
