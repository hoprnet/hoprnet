import { API_URL } from './env'
import { ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'
import { Message } from './message'
import { bounceBot } from './bouncebot'
import { randoBot } from './randobot'
import { SetupClient, getHoprAddress, sendMessage, getMessageStream  } from './utils'


const start = async () => {
  console.log(`Connecting to ${API_URL}`)
  const hoprAddress = await getHoprAddress()
  console.log(`My HOPR address is ${hoprAddress}`)

  // Adding bots
  bounceBot(hoprAddress);
  randoBot(hoprAddress);
}

start().catch(console.error)
