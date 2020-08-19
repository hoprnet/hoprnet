import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type { AutoCompleteResult } from './abstractCommand'
import { AbstractCommand, GlobalState } from './abstractCommand'
import CloseChannel from './closeChannel'
import Crawl from './crawl'
import ListCommands from './listCommands'
import ListConnectors from './listConnectors'
import ListOpenChannels from './listOpenChannels'
import OpenChannel from './openChannel'
import Ping from './ping'
import PrintAddress from './printAddress'
import PrintBalance from './printBalance'
import SendMessage from './sendMessage'
import StopNode from './stopNode'
import Version from './version'
import Tickets from './tickets'
import IncludeRecipient from './includeRecipient'
import readline from 'readline'

export const SPLIT_OPERAND_QUERY_REGEX: RegExp = /([\w\-]+)(?:\s+)?([\w\s\-.]+)?/

export class Commands {
  readonly commands: AbstractCommand[]
  private commandMap: Map<string, AbstractCommand>
  private state: GlobalState

  constructor(public node: Hopr<HoprCoreConnector>, rl?: readline.Interface) {
    this.state = {
      includeRecipient: false,
      aliases: new Map()
    }


    this.commands = [
      new CloseChannel(node),
      new Crawl(node),
      new ListCommands(() => this.commands),
      new ListConnectors(),
      new ListOpenChannels(node),
      new Ping(node),
      new PrintAddress(node),
      new StopNode(node),
      new Version(),
      new Tickets(node),
    ]

    if(rl) {
      this.commands.push(new OpenChannel(node, rl))
      this.commands.push(new SendMessage(node, rl))
      this.commands.push(new IncludeRecipient(node, rl))
    }
    this.commandMap = new Map()
    for (let command of this.commands) {
      if (this.commandMap.has(command.name())){
        throw new Error(`Duplicate commands for ${command}`)
      }
      this.commandMap.set(command.name(), command)
    }
  }

  public allCommands(): string[] {
    return Array.from(this.commandMap.keys())
  }

  public find(command: string): AbstractCommand {
    return this.commandMap.get(command.trim())
  }
  
  public async execute(message: string): Promise<boolean> {
    const [command, query]: (string | undefined)[] = message.trim().split(SPLIT_OPERAND_QUERY_REGEX).slice(1)

    if (command == null) {
      return true;
    }

    let cmd = this.find(command)
    
    if (cmd){
      await cmd.execute(query, this.state)
      return true
    }
    return false
  }

  public async autocomplete(message: string): Promise<AutoCompleteResult> {
    // If the line is empty, we show all possible commands as results.
    if (message == null || message == '') {
      return [this.allCommands(), message]
    }

    const [command, query]: (string | undefined)[] = message.trim().split(SPLIT_OPERAND_QUERY_REGEX).slice(1)
    const cmd = await this.find(command)
    if (cmd) {
      return cmd.autocomplete(query, message)
    }
    // Command not found - try assuming it's an incomplete command
    const hits = this.allCommands().reduce((acc: string[], name: string) => {
          if (name.startsWith(message)) {
            acc.push(name)
          }
          return acc
    }, [])

    if (hits.length > 0){
      return [hits, message]
    }

    // We did our best, lets just show all possible commands
    return [this.allCommands(), message]
  }
}
