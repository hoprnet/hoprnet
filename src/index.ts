import { API_URL, BOT_NAME } from './env'
import { getHoprAddress  } from './utils'
import { setupBot, Bot } from './bot'
import { setupPayDai, payDai } from './linkdrop'


const start = async () => {
  console.log(`Connecting to ${API_URL}`)
  const hoprAddress = await getHoprAddress()
  console.log(`My HOPR address is ${hoprAddress}`)

  let bot: Bot
  switch(BOT_NAME) {
    case 'randobot': 
      const { Randombot } = await import("./randobot")
      bot = new Randombot(hoprAddress)
      break
    case 'bouncerbot':
      const { Bouncebot } = await import("./bouncerbot")
      bot = new Bouncebot(hoprAddress)
      break
    case 'tweetbot':
      const { Tweetbot } = await import("./tweetbot")
      bot = new Tweetbot(hoprAddress)
      break
  }
  await setupPayDai(10)
  await setupBot(bot)
}

start().catch((err) => {
  console.error('Fatal Error:', err)
  process.exit()
})
