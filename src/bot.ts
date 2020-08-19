import { getMessageStream } from './utils'
import { IMessage, Message } from './message'
import { ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'

export interface Bot {
    botName: string
    address: string
    handleMessage(message: IMessage)
}

export async function setupBot(bot: Bot) {
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