import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import { OpenChannel } from './openChannel'
import PrintBalance from './printBalance'
import Withdraw from './withdraw'
import { Alias } from './alias'
import { AbstractCommand, GlobalState } from '../abstractCommand'

export const isError = (error: any): error is Error => {
  return error instanceof Error
}

export class CommandsV2 {
  readonly commands: AbstractCommand[]
  private commandMap: Map<string, AbstractCommand>
  public state: GlobalState

  constructor(public node: Hopr) {
    this.state = {
      aliases: new Map<string, PeerId>(),
      includeRecipient: false
    }

    this.commands = [new Alias(node), new PrintBalance(node), new Withdraw(node), new OpenChannel(node)]

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

  public async execute(log, message: string): Promise<string | void> {
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

    log(`${cmd}: Unknown command!`)
    return 'Unknown command!'
  }
}
