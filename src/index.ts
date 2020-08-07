import { API_URL } from './env'
import { bounceBot } from './bouncebot'
import { randoBot } from './randobot'
import { getHoprAddress  } from './utils'


const start = async () => {
  console.log(`Connecting to ${API_URL}`)
  const hoprAddress = await getHoprAddress()
  console.log(`My HOPR address is ${hoprAddress}`)

  // Adding bots
  bounceBot(hoprAddress);
  randoBot(hoprAddress);
}

start().catch((err) => {
  console.error('Fatal Error:', err)
  process.exit();
})
