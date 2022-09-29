import type { Command, CacheFunctions } from '../utils/command'
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
import EntryNodes from './entryNodes'

export default class Commands {
  private commandMap: Map<string, Command> = new Map()

  constructor(private api: API, private cache: CacheFunctions) {
    const commands: Command[] = [
      new Alias(this.api, this.cache),
      new Addresses(this.api, this.cache),
      new Balances(this.api, this.cache),
      new Sign(this.api, this.cache),
      new Peers(this.api, this.cache),
      new EntryNodes(this.api, this.cache),
      new Ping(this.api, this.cache),
      new Channels(this.api, this.cache),
      new OpenChannel(this.api, this.cache),
      new CloseChannel(this.api, this.cache),
      new SendMessage(this.api, this.cache),
      new Tickets(this.api, this.cache),
      new RedeemTickets(this.api, this.cache),
      new Withdraw(this.api, this.cache),
      new Settings(this.api, this.cache),
      new PeerInfo(this.api, this.cache),
      new Info(this.api, this.cache),
      new Version(this.api, this.cache)
    ]

    commands.push(new Help(this.api, this.cache, commands))

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
      // user is requesting usage
      if (query === 'help') {
        return log(cmd.usage())
      }

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
