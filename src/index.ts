import { API_URL, BOT_NAME, BOT_TIMESTAMP } from './env'
import { getHoprAddress  } from './utils'
import { setupBot, Bot } from './bot'
import { payDai } from './linkdrop'


const start = async () => {
  console.log(`Connecting to ${API_URL}`)
  const hoprAddress = await getHoprAddress()
  const timestamp = BOT_TIMESTAMP ? new Date(+BOT_TIMESTAMP) : new Date(Date.now());
  console.log(`My HOPR address is ${hoprAddress}`)

  let bot: Bot
  switch(BOT_NAME) {
    case 'randobot': 
      const { Randombot } = await import("./randobot")
      bot = new Randombot(hoprAddress, timestamp)
      break
    case 'bouncerbot':
      const { Bouncebot } = await import("./bouncerbot")
      bot = new Bouncebot(hoprAddress, timestamp)
      break
    case 'tweetbot':
      const { Tweetbot } = await import("./tweetbot")
      bot = new Tweetbot(hoprAddress, timestamp)
      break
  }
  await setupBot(bot)
}

start().catch((err) => {
  console.error('Fatal Error:', err)
  process.exit()
})
