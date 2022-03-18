import type { Command } from '../utils/command'
import API from '../utils/api'
// commands
import Alias from './alias'
import Addresses from './addresses'
import Balances from './balances'
import Sign from './sign'
import Peers from './peers'
import Ping from './ping'
import Channels from './channels'
import OpenChannel from './openChannel'
import CloseChannel from './closeChannel'
import SendMessage from './sendMessage'
import Tickets from './tickets'
import RedeemTickets from './redeemTickets'
import Withdraw from './withdraw'
import Settings from './settings'
import PeerInfo from './peerInfo'
import Info from './info'
import Version from './version'
import Help from './help'

export default class Commands {
  private commandMap: Map<string, Command> = new Map()

  constructor(private api: API, getCachedAliases: () => Record<string, string>) {
    const extra = { getCachedAliases }

    const commands: Command[] = [
      new Alias(this.api, extra),
      new Addresses(this.api, extra),
      new Balances(this.api, extra),
      new Sign(this.api, extra),
      new Peers(this.api, extra),
      new Ping(this.api, extra),
      new Channels(this.api, extra),
      new OpenChannel(this.api, extra),
      new CloseChannel(this.api, extra),
      new SendMessage(this.api, extra),
      new Tickets(this.api, extra),
      new RedeemTickets(this.api, extra),
      new Withdraw(this.api, extra),
      new Settings(this.api, extra),
      new PeerInfo(this.api, extra),
      new Info(this.api, extra),
      new Version(this.api, extra)
    ]

    commands.push(new Help(this.api, extra, commands))

    for (const command of commands) {
      if (this.commandMap.has(command.name())) {
        throw new Error(`Duplicate commands for ${command}`)
      }
      this.commandMap.set(command.name(), command)
    }
  }

  public allCommands(): string[] {
    return Array.from(this.commandMap.keys())
  }

  public async execute(log: (msg: string) => void, userInput: string): Promise<void> {
    const split: string[] = userInput.trim().split(' ')
    const [cmdName, ...params] = split
    const query = params.join(' ')

    if (!cmdName) return log(`Command not provided!`)

    log('> ' + userInput)
    let cmd = this.commandMap.get(cmdName)

    if (cmd) {
      try {
        return await cmd.execute(log, query)
      } catch (error: any) {
        console.error(error)
        return log(`${cmdName}: Unexpected error executing command.\n${cmd.usage()}`)
      }
    }

    return log(`${cmdName}: Unknown command!`)
  }
}
