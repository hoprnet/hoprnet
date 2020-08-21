import type PeerId from 'peer-id'

export type AutoCompleteResult = [string[], string] 
export const emptyAutoCompleteResult = (line: string):AutoCompleteResult => [[''], line]

export type CommandResponse = string | void

export type GlobalState = {
  includeRecipient: boolean
  aliases: Map<string, PeerId>
}

// REPL Command
export abstract class AbstractCommand {
  // The command, for example 'ping' or 'foo'
  abstract name(): string

  // A help string describing the command
  abstract help(): string

  // Run the command with optional argument
  abstract execute(query: string, state: GlobalState): CommandResponse | Promise<CommandResponse>

  async autocomplete(query: string, line: string, state: GlobalState): Promise<AutoCompleteResult> {
    return emptyAutoCompleteResult(line) // default is no further results, end the query there, based on the whole line
  }

  // In most cases we are autocompleting by filtering results with a prefix
  // NB. Because we need to pass the whole line back, this assumes that the
  // entire query after the command name is being handled.
  protected _autocompleteByFiltering(query: string, allResults: string[], line: string): AutoCompleteResult {
    if (allResults.length == 0){ 
      return emptyAutoCompleteResult(line)
    }
    const response = x => `${this.name()} ${x}`
    if (!query){ // If the query is an empty string, we show all options.
      return [allResults.map(response), line]
    }
    let filtered = allResults.filter(x => x.startsWith(query))
    if (filtered.length == 0){
      return emptyAutoCompleteResult(line) // Readline can't handle empty results
    }
    return [filtered.map(response), line]
  }
}
