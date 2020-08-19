import type PeerId from 'peer-id'

export type AutoCompleteResult = [string[], string] 

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

  async autocomplete(query: string, line: string): Promise<AutoCompleteResult | undefined> {
    return [[''], line] // default is no further results, end the query there, based on the whole line
  }
}
