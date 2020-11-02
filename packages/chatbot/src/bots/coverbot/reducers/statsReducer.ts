import debug from 'debug'
import Instruction from '../classes/instruction'
import { IMessage } from '../../../message/message'
import { HOPR_ENVIRONMENT } from '../../../utils/env'
import { BotCommands, StatsSubCommands } from '../types/commands'
import { StatsStateResponses } from '../responses'
import { Coverbot } from '..'

const log = debug('hopr-chatbot:reducers:stats')
const error = debug('hopr-chatbot:reducers:stats:error')

export async function statsReducer(this: Coverbot, instructionWrapper: Instruction, message: IMessage) {
  log(
    `- statsReducer | Starting statsReducer with instruction ${instructionWrapper.toString()} and message ${
      message.text
    } from ${message.from}`,
  )
  const state = (await this.database.getTable(HOPR_ENVIRONMENT, 'state')) || {}
  log(`- statsReducer | ${BotCommands.stats} command :: retrieving state with value ${JSON.stringify(state)}`)
  switch (instructionWrapper.subcommand) {
    case StatsSubCommands.connected: {
      log(`- statsReducer | ${BotCommands.stats} command :: ${StatsSubCommands.connected} subcommand received`)
      log(
        `- statsReducer | ${BotCommands.stats} command :: ${StatsSubCommands.connected} subcommand retrieving state from snapshot with value ${state}`,
      )
      const connectedNodes: number = state && state.connectedNodes ? state.connectedNodes : 0
      log(
        `- statsReducer | ${BotCommands.stats} command :: ${StatsSubCommands.connected} subcommand retrieving connected nodes from state with value ${connectedNodes}`,
      )
      this._sendMessageFromBot(
        message.from,
        (StatsStateResponses[StatsSubCommands.connected] as Function)(connectedNodes),
      ).catch((err) => {
        error(`Trying to send ${BotCommands.stats} message to ${message.from} failed.`, err)
      })
      break
    }
    default: {
      log(`- statsReducer | ${BotCommands.stats} command :: subcommand not understood`)
      this._sendMessageFromBot(message.from, StatsStateResponses[StatsSubCommands.help] as string).catch((err) => {
        error(`Trying to send ${StatsSubCommands.help} message to ${message.from} failed.`, err)
      })
    }
  }
}
