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
import HoprFetcher from '../fetch'
import Addresses from './addresses'

export class Commands {
  readonly commands: AbstractCommand[]
  private commandMap: Map<string, AbstractCommand>
  readonly hoprFetcher: HoprFetcher

  constructor(apiPort: number, apiToken: string) {
    this.hoprFetcher = new HoprFetcher(apiPort, apiToken)

    this.commands = [
      new Addresses(this.hoprFetcher),
      new Alias(this.hoprFetcher),
      new CloseChannel(this.hoprFetcher),
      new Info(this.hoprFetcher),
      new ListConnectedPeers(this.hoprFetcher),
      new ListCommands(this.hoprFetcher, () => this.commands),
      new ListOpenChannels(this.hoprFetcher),
      new Ping(this.hoprFetcher),
      new PrintAddress(this.hoprFetcher),
      new PrintBalance(this.hoprFetcher),
      new RedeemTickets(this.hoprFetcher),
      new Sign(this.hoprFetcher),
      new Version(this.hoprFetcher),
      new Tickets(this.hoprFetcher),
      new SendMessage(this.hoprFetcher),
      new Settings(this.hoprFetcher),
      new Withdraw(this.hoprFetcher),
      new OpenChannel(this.hoprFetcher)
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

  public async execute(log: string, message: string): Promise<void> {
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
