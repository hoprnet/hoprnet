import { AbstractCommand } from './abstractCommand'
import CloseChannel from './closeChannel'
import ListCommands from './listCommands'
import ListOpenChannels from './listOpenChannels'
import ListConnectedPeers from './listConnected'
import { OpenChannel } from './openChannel'
import Ping from './ping'
import Sign from './sign'
import PrintAddress from './printAddress'
import PrintBalance from './printBalance'
import { SendMessage } from './sendMessage'
import Version from './version'
import Tickets from './tickets'
import RedeemTickets from './redeemTickets'
import Settings from './settings'
import Withdraw from './withdraw'
import { Alias } from './alias'
import { Info } from './info'
import Addresses from './addresses'

export class Commands {
  readonly commands: AbstractCommand[]
  private commandMap: Map<string, AbstractCommand>

  constructor() {
    this.commands = [
      new Addresses(),
      new Alias(),
      new CloseChannel(),
      new Info(),
      new ListConnectedPeers(),
      new ListCommands(() => this.commands),
      new ListOpenChannels(),
      new Ping(),
      new PrintAddress(),
      new PrintBalance(),
      new RedeemTickets(),
      new Sign(),
      new Version(),
      new Tickets(),
      new SendMessage(),
      new Settings(),
      new Withdraw(),
      new OpenChannel(),
    ]

    this.commandMap = new Map()
    for (let command of this.commands) {
      if (this.commandMap.has(command.name())) {
        throw new Error(`Duplicate commands for ${command}`)
      }
      this.commandMap.set(command.name(), command)
    }
  }

  public allCommands(): string[] {
    return Array.from(this.commandMap.keys())
  }

  public find(command: string): AbstractCommand | undefined {
    return this.commandMap.get(command.trim())
  }

  public async execute(log: (string), message: string): Promise<void> {
    const split: (string | undefined)[] = message.trim().split(/\s+/)
    const command = split[0]
    const query = split.slice(1).join(' ')

    if (command == null) {
      return undefined
    }

    let cmd = this.find(command)

    if (cmd) {
      try {
        return await cmd.execute(log, query || '')
      } catch (err) {
        return log(`${cmd} execution failed with error: ${err.message}`)
      }
    }

    return log(`${cmd}: Unknown command!`)
  }
}
