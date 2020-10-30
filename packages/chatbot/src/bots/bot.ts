import {IMessage, Message} from '../message/message'
import wait from 'wait-for-stuff'
import Core from '../lib/hopr/core'
import debug from 'debug'

const log = debug('hopr-chatbot:bot')
const error = debug('hopr-chatbot:bot:error')

export interface Bot {
  botName: string
  address: string
  timestamp: Date
  twitterTimestamp: Date
  automaticResponse?: boolean
  handleMessage(message: IMessage)
}

const listen = async (bot: Bot, node: Core) => {
  log('- listen | Ready to listen to my HOPR node')

  node.events.on('message', (decoded: Buffer) => {
    const message = new Message(new Uint8Array(decoded))
    const parsedMessage = message.toJson()
    if (!parsedMessage.from) {
      error('- listen:message | Someone forgot to includeRecipient...')
    }
    const response = bot.handleMessage.call(bot, parsedMessage)
    log('- listen:message | Bot Response', response)
    bot.automaticResponse &&
      node.send({
        peerId: parsedMessage.from,
        payload: Message.fromJson({from: bot.address, text: ` ${response}`}).toU8a(),
        intermediatePeerIds: [],
      })
  })
}

export async function setupBot(bot: Bot, node: Core) {
  log(`- setupBot | Starting bot at ${bot.timestamp} for node ${await node.address('hopr')}`)
  log(`- setupBot | Listening to Tweets created after ${bot.twitterTimestamp}`)
  wait.for.date(bot.timestamp)
  await listen(bot, node)
}
