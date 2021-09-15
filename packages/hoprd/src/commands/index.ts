import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import { AbstractCommand, GlobalState } from './abstractCommand'
import FundChannel from './fundChannel'
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
import StopNode from './stopNode'
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
  private state: GlobalState

  constructor(public node: Hopr) {
    this.state = {
      aliases: new Map<string, PeerId>(),
      includeRecipient: false
    }

    this.commands = [
      new Addresses(node),
      new Alias(node),
      new CloseChannel(node),
      new Info(node),
      new ListConnectedPeers(node),
      new ListCommands(() => this.commands),
      new ListOpenChannels(node),
      new Ping(node),
      new PrintAddress(node),
      new PrintBalance(node),
      new RedeemTickets(node),
      new Sign(node),
      new StopNode(node),
      new Version(node),
      new Tickets(node),
      new SendMessage(node),
      new Settings(node),
      new Withdraw(node),
      new OpenChannel(node),
      new FundChannel(node)
    ]

    this.commandMap = new Map()
    for (let command of this.commands) {
      if (this.commandMap.has(command.name())) {
        throw new Error(`Duplicate commands for ${command}`)
      }
      this.commandMap.set(command.name(), command)
    }
  }

  public setState(settings: any) {
    this.state = settings
  }

  public allCommands(): string[] {
    return Array.from(this.commandMap.keys())
  }

  public find(command: string): AbstractCommand | undefined {
    return this.commandMap.get(command.trim())
  }

  public async execute(log, message: string): Promise<void> {
    const split: (string | undefined)[] = message.trim().split(/\s+/)
    const command = split[0]
    const query = split.slice(1).join(' ')

    if (command == null) {
      return undefined
    }

    let cmd = this.find(command)

    if (cmd) {
      return await cmd.execute(log, query || '', this.state)
    }

    return log(`${cmd}: Unknown command!`)
  }
}
