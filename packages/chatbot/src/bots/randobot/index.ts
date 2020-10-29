import {Bot} from '../bot'
import {IMessage} from '../../message/message'
import {generateRandomSentence} from '../../utils/utils'
import debug from 'debug'

const log = debug('hopr-chatbot:randobot')

export class Randombot implements Bot {
  automaticResponse: boolean
  botName: string
  address: string
  timestamp: Date
  twitterTimestamp: Date

  constructor(address: string, timestamp: Date, twitterTimestamp: Date) {
    this.automaticResponse = true
    this.address = address
    this.timestamp = timestamp
    this.twitterTimestamp = twitterTimestamp
    this.botName = '🃏 Randobot'
    log(`- constructor | ${this.botName} has been added`)
  }

  handleMessage(message: IMessage) {
    log(`- handleMessage | ${this.botName} <- ${message.from}: ${message.text}`)
    return generateRandomSentence()
  }
}
