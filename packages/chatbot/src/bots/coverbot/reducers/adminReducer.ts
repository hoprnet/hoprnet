import debug from 'debug'
import Instruction from '../classes/instruction'
import { IMessage } from '../../../message/message'
import { COVERBOT_ADMIN_MODE } from '../../../utils/env'
import { NodeStateResponses, AdminStateResponses } from '../responses'
import { NodeStates } from '../types/states'
import { BotCommands, AdminSubCommands } from '../types/commands'
import { Coverbot } from '..'


const log = debug('hopr-chatbot:reducers:admin')
const error = debug('hopr-chatbot:reducers:admin:error')

export async function adminReducer(this: Coverbot, instructionWrapper: Instruction, message: IMessage) {
  log(`- adminReducer | Starting adminReducer with instruction ${instructionWrapper.toString()} and message ${message.text} from ${message.from}`)
  if (!COVERBOT_ADMIN_MODE) {
    return this._sendMessageFromBot(message.from, NodeStateResponses[NodeStates.adminModeDisabled]).catch((err) => {
      error(`Trying to send ${NodeStates.adminModeDisabled} message to ${message.from} failed.`, err)
    })
  } else {
    log(`- adminReducer | ${BotCommands.admin} command :: allowed to go forward`)
    switch (instructionWrapper.subcommand) {
      case AdminSubCommands.help: {
        log(`- adminReducer | ${BotCommands.admin} command :: ${AdminSubCommands.help} subcommand received`)
        this._sendMessageFromBot(message.from, AdminStateResponses[AdminSubCommands.help]).catch((err) => {
          error(`Trying to send ${AdminSubCommands.help} message to ${message.from} failed.`, err)
        })
        break;
      }
      case AdminSubCommands.coverTrafficCycle: {
        log(`- adminReducer | ${BotCommands.admin} command :: ${AdminSubCommands.coverTrafficCycle} subcommand received`)
        this._sendMessageFromBot(message.from, AdminStateResponses[AdminSubCommands.coverTrafficCycle]).catch((err) => {
          error(`Trying to send ${AdminSubCommands.coverTrafficCycle} message to ${message.from} failed.`, err)
        })
        log(`- adminReducer | ${BotCommands.admin} command :: ${AdminSubCommands.coverTrafficCycle} subcommand :: starting verification cycle`)
        await this._startCycles.call(this);
        log(`- adminReducer | ${BotCommands.admin} command :: ${AdminSubCommands.coverTrafficCycle} subcommand :: completed verification cycle`)
        break;
      }
      case AdminSubCommands.saveState: {
        log(`- adminReducer | ${BotCommands.admin} command :: ${AdminSubCommands.saveState} subcommand received`)
        this._sendMessageFromBot(message.from, AdminStateResponses[AdminSubCommands.saveState]).catch((err) => {
          error(`Trying to send ${AdminSubCommands.saveState} message to ${message.from} failed.`, err)
        })
        log(`- adminReducer | ${BotCommands.admin} command :: ${AdminSubCommands.saveState} subcommand :: starting saving state`)
        await this._dumpData()
        log(`- adminReducer | ${BotCommands.admin} command :: ${AdminSubCommands.saveState} subcommand :: completed saving state`)
        break;
      }
      default: {
        log(`- handleMessage | admin command :: subcommand not understood`)
        this._sendMessageFromBot(message.from, AdminStateResponses[AdminSubCommands.help]).catch((err) => {
          error(`Trying to send ${AdminSubCommands.help} message to ${message.from} failed.`, err)
        })
      }
    }
  }
}


