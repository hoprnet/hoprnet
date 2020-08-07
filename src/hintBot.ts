import { getMessageStream, sendMessage } from './utils'
import { ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'
import { Message } from './message'


const directory = {}

const messages = [
  'Psst',
  'Wanna get in? BouncerBot is tough, but dumb.',
  'Just tweet about #HOPRGAMES, including your HOPR address, and I’ll make sure you get on the list.'
] 

export const hintBot = async (hoprAddress) => {
  const botName = '👀 Hintbot (v2)'
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

        let response;
        /*
        * We only respond if we receive a message with the word hintbot.
        * Unlike other bots, hintbot requires an address of an address
        * they should give the hint to.
        */
        if (message.text.match(/hintbot?$/i)) {
          sendMessage(message.from, {
            from: hoprAddress,
            text: ` ${response}`,
          })
        }
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
