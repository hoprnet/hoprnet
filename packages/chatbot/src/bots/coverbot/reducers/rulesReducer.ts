import debug from 'debug'
import { IMessage } from 'src/message/message';
import { Coverbot } from '..';
import { BotResponses } from '../responses';
import { BotCommands } from '../types/commands';


const log = debug('hopr-chatbot:reducers:rules')
const error = debug('hopr-chatbot:reducers:rules:error')

export async function rulesReducer(this: Coverbot, message: IMessage) {
  log(`- rulesReducer | Starting rulesReducer with message ${message.text} from ${message.from}`)
  this._sendMessageFromBot(message.from, BotResponses[BotCommands.rules]).catch((err) => {
    error(`Trying to send ${BotCommands.rules} message to ${message.from} failed.`, err)
  })
}