import { getMessageStream, sendMessage } from '../utils'
import { ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'
import { Message } from '../message'
import { generateRandomSentence } from '../utils'


export default async (hoprAddress) => {
  const botName = '🃏 Randobot'
  console.log(`${botName} has been added`);

  const { client, stream } = await getMessageStream()

  stream
    .on('data', (data) => {
      try {
        const [messageBuffer] = data.array
        const res = new ListenResponse()
        res.setPayload(messageBuffer)

        const message = new Message(res.getPayload_asU8()).toJson()
        console.log(`${botName} <- ${message.from}: ${message.text}`)

        sendMessage(message.from, {
          from: hoprAddress,
          text: ` ${botName} says ${generateRandomSentence()}`,
        })
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
