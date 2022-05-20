import type Hopr from '@hoprnet/hopr-core'
import type { StateOps } from '../types.mjs'
import { AbstractCommand } from './abstractCommand.mjs'
import FundChannel from './fundChannel.mjs'
import CloseChannel from './closeChannel.mjs'
import ListCommands from './listCommands.mjs'
import ListOpenChannels from './listOpenChannels.mjs'
import ListConnectedPeers from './listConnected.mjs'
import { OpenChannel } from './openChannel.mjs'
import Ping from './ping.mjs'
import Sign from './sign.mjs'
import PrintAddress from './printAddress.mjs'
import PrintBalance from './printBalance.mjs'
import { SendMessage } from './sendMessage.mjs'
import StopNode from './stopNode.mjs'
import Version from './version.mjs'
import Tickets from './tickets.mjs'
import RedeemTickets from './redeemTickets.mjs'
import Settings from './settings.mjs'
import Withdraw from './withdraw.mjs'
import { Alias } from './alias.mjs'
import { Info } from './info.mjs'
import Addresses from './addresses.mjs'

export class Commands {
  readonly commands: AbstractCommand[]
  private commandMap: Map<string, AbstractCommand>

  constructor(public node: Hopr, public stateOps: StateOps) {
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
      try {
        return await cmd.execute(log, query || '', this.stateOps)
      } catch (err) {
        return log(`${cmd} execution failed with error: ${err.message}`)
      }
    }

    return log(`${cmd}: Unknown command!`)
  }
}
