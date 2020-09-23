import { IMessage, Message } from '../message/message'
import wait from 'wait-for-stuff'
import Core from '../lib/hopr/core'

export interface Bot {
  botName: string
  address: string
  timestamp: Date
  twitterTimestamp: Date
  handleMessage(message: IMessage)
}

const listen = async (bot: Bot, node: Core) => {
  console.log('[ Chatbot ] listen | Ready to listen to my HOPR node')

  node.events.on('message', (decoded: Buffer) => {
    const message = new Message(new Uint8Array(decoded))
    const parsedMessage = message.toJson()
    const response = bot.handleMessage.call(bot, parsedMessage)
    console.log('[ Chatbot ] listen:message | Bot Response', response)
    node.send({
      peerId: parsedMessage.from,
      payload: Message.fromJson({ from: bot.address, text: ` ${response}` }).toU8a(),
      intermediatePeerIds: [],
    })
  })
}

export async function setupBot(bot: Bot, node: Core) {
  console.log(`Starting bot at ${bot.timestamp}`)
  console.log(`Listening to Tweets created after ${bot.twitterTimestamp}`)
  wait.for.date(bot.timestamp)
  await listen(bot, node)
}
