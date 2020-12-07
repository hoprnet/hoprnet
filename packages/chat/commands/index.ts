import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import { AutoCompleteResult } from './abstractCommand'
import { AbstractCommand, GlobalState, CommandResponse } from './abstractCommand'
import CloseChannel from './closeChannel'
import ListCommands from './listCommands'
// import ListConnectors from './listConnectors'
import ListOpenChannels from './listOpenChannels'
import ListConnectedPeers from './listConnected'
import { OpenChannelFancy, OpenChannel } from './openChannel'
import Ping from './ping'
import PrintAddress from './printAddress'
import PrintBalance from './printBalance'
import { SendMessageFancy, SendMessage } from './sendMessage'
import { MultiSendMessage } from './multisend'
import StopNode from './stopNode'
import Version from './version'
import Tickets from './tickets'
import RedeemTickets from './redeemTickets'
import Settings from './settings'
import Withdraw from './withdraw'
import TraverseChannels from './traverseChannels'
import readline from 'readline'
import { Alias } from './alias'
import { Info } from './info'
import { SetStrategy } from './strategy'

export class Commands {
  readonly commands: AbstractCommand[]
  private commandMap: Map<string, AbstractCommand>
  private state: GlobalState

  constructor(public node: Hopr<HoprCoreConnector>, rl?: readline.Interface) {
    this.state = {
      aliases: new Map<string, PeerId>(),
      includeRecipient: false,
      routing: 'direct',
      routingPath: []
    }

    this.commands = [
      new CloseChannel(node),
      new Info(node),
      new ListCommands(() => this.commands),
      // new ListConnectors(),
      new ListConnectedPeers(node),
      new ListOpenChannels(node),
      new Ping(node),
      new PrintAddress(node),
      new PrintBalance(node),
      new StopNode(node),
      new Version(),
      new Tickets(node),
      new RedeemTickets(node),
      new Settings(),
      new Alias(node),
      new TraverseChannels(node),
      new Withdraw(node),
      new SetStrategy(node)
    ]

    if (rl) {
      this.commands.push(new OpenChannelFancy(node, rl))
      this.commands.push(new SendMessageFancy(node, rl))
      this.commands.push(new MultiSendMessage(node, rl))
    } else {
      this.commands.push(new OpenChannel(node))
      this.commands.push(new SendMessage(node))
    }

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

  public async execute(message: string): Promise<CommandResponse> {
    const split: (string | undefined)[] = message.trim().split(/\s+/)
    const command = split[0]
    const query = split.slice(1).join(' ')

    if (command == null) {
      return undefined
    }

    let cmd = this.find(command)

    if (cmd) {
      return await cmd.execute(query || '', this.state)
    }

    return 'Unknown command!'
  }

  public async autocomplete(message: string): Promise<AutoCompleteResult> {
    // If the line is empty, we show all possible commands as results.
    if (!message) {
      return [this.allCommands(), message]
    }

    const [command, query]: (string | undefined)[] = message.trim().split(/\s+/).slice(0)
    const cmd = this.find(command)
    if (cmd && typeof cmd.autocomplete !== 'undefined') {
      return cmd.autocomplete(query, message, this.state)
    }
    // Command not found - try assuming it's an incomplete command
    const hits = this.allCommands().reduce((acc: string[], name: string) => {
      if (name.startsWith(message)) {
        acc.push(name)
      }
      return acc
    }, [])

    if (hits.length > 0) {
      return [hits, message]
    }

    // We did our best, lets just show all possible commands
    return [this.allCommands(), message]
  }
}
