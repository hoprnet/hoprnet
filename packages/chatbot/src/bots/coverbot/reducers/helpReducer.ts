import debug from 'debug'
import { IMessage } from 'src/message/message'
import { Coverbot } from '..'
import { BotResponses } from '../responses'
import { BotCommands } from '../types/commands'

const log = debug('hopr-chatbot:reducers:help')
const error = debug('hopr-chatbot:reducers:help:error')

export async function helpReducer(this: Coverbot, message: IMessage) {
  log(`- helpReducer | Starting helpReducer with message ${message.text} from ${message.from}`)
  this._sendMessageFromBot(message.from, BotResponses[BotCommands.help]).catch((err) => {
    error(`Trying to send ${BotCommands.help} message to ${message.from} failed.`, err)
  })
}
