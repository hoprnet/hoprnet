import type PeerId from 'peer-id'
import { styleValue } from '../utils'

export type AutoCompleteResult = [string[], string]
export const emptyAutoCompleteResult = (line: string): AutoCompleteResult => [[''], line]
export type CommandResponse = string | void

export type GlobalState = {
  aliases: Map<string, PeerId>
  includeRecipient: boolean
  routing: 'direct' | 'manual'
  routingPath: PeerId[]
}

// REPL Command
export abstract class AbstractCommand {
  // The command, for example 'ping' or 'foo'
  abstract name(): string

  // A help string describing the command
  abstract help(): string

  // Run the command with optional argument
  abstract execute(query: string, state: GlobalState): CommandResponse | Promise<CommandResponse>

  async autocomplete(_query: string, line: string, _state: GlobalState): Promise<AutoCompleteResult> {
    return emptyAutoCompleteResult(line) // default is no further results, end the query there, based on the whole line
  }

  // In most cases we are autocompleting by filtering results with a prefix
  // NB. Because we need to pass the whole line back, this assumes that the
  // entire query after the command name is being handled.
  protected _autocompleteByFiltering(query: string, allResults: string[], line: string): AutoCompleteResult {
    if (allResults.length == 0) {
      return emptyAutoCompleteResult(line)
    }
    const response = (x: string) => `${this.name()} ${x}`
    if (!query) {
      // If the query is an empty string, we show all options.
      return [allResults.map(response), line]
    }
    let filtered = allResults.filter((x) => x.startsWith(query))
    if (filtered.length == 0) {
      return emptyAutoCompleteResult(line) // Readline can't handle empty results
    }
    return [filtered.map(response), line]
  }

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
