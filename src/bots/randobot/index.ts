import { Bot } from '../bot'
import { IMessage } from '../../message/message'
import { generateRandomSentence } from '../../utils/utils'

export class Randombot implements Bot {
  botName: string
  address: string
  timestamp: Date
  twitterTimestamp: Date

  constructor(address: string, timestamp: Date, twitterTimestamp: Date) {
    this.address = address
    this.timestamp = timestamp
    this.twitterTimestamp = twitterTimestamp
    this.botName = '🃏 Randobot'
    console.log(`${this.botName} has been added`)
  }

  handleMessage(message: IMessage) {
    console.log(`${this.botName} <- ${message.from}: ${message.text}`)
    return generateRandomSentence()
  }
}
