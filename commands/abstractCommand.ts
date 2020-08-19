
export type AutoCompleteResult = [string[], string] 

export type GlobalState = {
  includeRecipient: boolean
  aliases: Map<string, string>
}

// REPL Command
export abstract class AbstractCommand {
  // The command, for example 'ping' or 'foo'
  abstract name(): string

  // A help string describing the command
  abstract help(): string

  // Run the command with optional argument
  abstract execute(query: string, state: GlobalState): void | Promise<void>

  async autocomplete(query: string, line: string): Promise<AutoCompleteResult | undefined> {
    return [[''], line] // default is no further results, end the query there, based on the whole line
  }
}
