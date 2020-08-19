import { sendMessage } from '../utils'
import { Bot } from '../bot'
import { IMessage } from '../message'
import { generateRandomSentence } from '../utils'


export class Randombot implements Bot {
  botName: string
  address: string

  constructor(address: string) {
    this.address = address
    this.botName = '🃏 Randobot'
    console.log(`${this.botName} has been added`)
  }

  handleMessage(message: IMessage) {
    console.log(`${this.botName} <- ${message.from}: ${message.text}`)
    sendMessage(message.from, {
      from: this.address,
      text: ` ${this.botName} says ${generateRandomSentence()}`,
    })
  }
}
