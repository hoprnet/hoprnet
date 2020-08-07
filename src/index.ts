import getopts from 'getopts'
import { API_URL } from './env'
import { bouncerBot } from './bouncerbot'
import { getHoprAddress  } from './utils'

const options = getopts(process.argv.slice(2), {
  string: ['bot'],
  alias: {
    b: 'bot',
  },
  default: {
    bot: 'randobot',
  },
})

const start = async () => {
  console.log(`Connecting to ${API_URL}`)
  const hoprAddress = await getHoprAddress()
  console.log(`My HOPR address is ${hoprAddress}`)

  console.log('Options', options);

  // Adding bots
  bouncerBot(hoprAddress);
  // randoBot(hoprAddress);
}

start().catch((err) => {
  console.error('Fatal Error:', err)
  process.exit();
})
