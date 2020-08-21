import { getMessageStream } from './utils'
import { IMessage, Message } from './message'
import { ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'
import wait from 'wait-for-stuff'

export interface Bot {
    botName: string
    address: string
    timestamp: Date
    twitterTimestamp: Date
    handleMessage(message: IMessage)
}

const listen = async (bot: Bot) => {
  console.log('Ready to listen to my HOPR node');
  const { client, stream } = await getMessageStream()
  stream
  .on('data', (data) => {
    try {
      const [messageBuffer] = data.array
      const res = new ListenResponse()
      res.setPayload(messageBuffer)
      const message = new Message(res.getPayload_asU8()).toJson()
      bot.handleMessage(message)
    } catch (err) {
      console.error(err)
    }
  })
  .on('error', (err) => {
    console.error(err)
  })
  .on('end', () => {
    client.close()
  })
}

export async function setupBot(bot: Bot) {
  console.log(`Starting bot at ${bot.timestamp}`)
  console.log(`Listening to Tweets created after ${bot.twitterTimestamp}`)
  wait.for.date(bot.timestamp);
  await listen(bot);
} 