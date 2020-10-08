import debug from 'debug'
import { BotCommands, VerifySubCommands, StatsSubCommands } from './state';


const log = debug('hopr-chatbot:instruction')
const error = debug('hopr-chatbot:instruction:error')

function isPropertyValue<T>(object: T, possibleValue: any): possibleValue is T[keyof T] {
  return Object.values(object).includes(possibleValue);
}

export default class Instruction {
  command: BotCommands
  subcommand: VerifySubCommands | StatsSubCommands
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
  enterInput(input: string) {
    if (!this.subcommand) {
      log(`- enterInput | Subcommand undefined, entering subcommand definition`)
      if (isPropertyValue(VerifySubCommands, input)) {
        log(`- enterInput | Subcommand ${input} accepted as a verification subcommand`)
        this.subcommand = input;
      } else if (isPropertyValue(StatsSubCommands, input)) {
        log(`- enterInput | Subcommand ${input} accepted as a stats subcommand`)
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