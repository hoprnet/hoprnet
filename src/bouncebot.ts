import { getMessageStream, sendMessage } from './utils'
import { ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'
import { Message } from './message'


const directory = {}

const messages = [
  'Oh, you here for the DAI party?',
  'I see. I don’t see you in the guest list. Only people on the guest list get past me. Get out of here.',
  'I don’t know what to tell you. If you ain’t on the list, you ain’t coming in',
  'This party’s for social media influencers only. Scram',
  'I don’t know what to tell you. I’m sure if you hang around long enough, someone will help you out'
] 

export const bounceBot = async (hoprAddress) => {
  const botName = '🥊 Bouncebot (v2)'
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
        if (message.text.match(/party?$/i)) {
          // Bounce bot gets messages and stores how many times has been reached.
          directory[message.from] = (directory[message.from] || 0) + 1
          response = directory[message.from] > messages.length ?
            'Stop messaging me until you get into the guest list. Maybe wait in line?' :
            messages[directory[message.from] - 1]
        } else {
          response = 'What do you want? I don’t understand...'
        }

        sendMessage(message.from, {
          from: hoprAddress,
          text: ` ${response}`,
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
