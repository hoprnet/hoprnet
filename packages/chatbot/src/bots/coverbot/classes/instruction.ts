import debug from 'debug'
import { BotCommands, AvailableSubCommands, VerifySubCommands, StatsSubCommands, AdminSubCommands } from '../types/commands';


const log = debug('hopr-chatbot:instruction')
const error = debug('hopr-chatbot:instruction:error')

function isPropertyValue<T>(object: T, possibleValue: any): possibleValue is T[keyof T] {
  return Object.values(object).includes(possibleValue);
}

export default class Instruction {
  command: BotCommands
  subcommand: AvailableSubCommands
  content: string
  constructor(maybeCommand: string) {
    log(`- constructor | Creating instruction w/maybecommand ${maybeCommand}`)
    if (isPropertyValue(BotCommands, maybeCommand)) {
      log(`- constructor | Command ${maybeCommand} accepted`)
      this.command = maybeCommand
    } else {
      error(`- constructor | Command ${maybeCommand} rejected as invalid`)
      throw new Error(`${maybeCommand} isn‘t a valid command`)
    }
  }
  toString() {
      return `${this.command} ${this.subcommand} ${this.content}`
  }
  enterInput(input: string) {
    if (!this.subcommand) {
      log(`- enterInput | Subcommand undefined, entering subcommand definition`)
      if (isPropertyValue(VerifySubCommands, input)) {
        log(`- enterInput | Subcommand ${input} accepted as a verification subcommand`)
        this.subcommand = input;
      } else if (isPropertyValue(StatsSubCommands, input)) {
        log(`- enterInput | Subcommand ${input} accepted as a stats subcommand`)
        this.subcommand = input;
      } else if (isPropertyValue(AdminSubCommands, input)) {
        log(`- enterInput | Subcommand ${input} accepted as a admin subcommand`)
        this.subcommand = input;
      } else {
        error(`- enterInput | Subcommand ${input} rejected as invalid`)
        throw new Error(`${input} isn‘t a valid subcommand`)
      }
    } else if (!this.content) {
      log(`- enterInput | Subcommand defined, entering subcommand content`)
      this.content = input;
    } else {
      error(`- enterInput | Too many arguments given as an instruction`)
      throw new Error(`Too many arguments given`)
    }
  }
}