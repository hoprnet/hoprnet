import { API_URL, BOT_NAME, BOT_TIMESTAMP, TWITTER_TIMESTAMP } from './env'
import { getHoprAddress } from './utils'
import { setupBot, Bot } from './bot'
import { payDai } from './linkdrop'
import Core from './core'

const start = async () => {
  const node = await new Core()
  await node.start();
  const hoprAddress = await node.address('hopr');

  const timestamp = BOT_TIMESTAMP ? new Date(+BOT_TIMESTAMP) : new Date(Date.now())
  const twitterTimestamp = TWITTER_TIMESTAMP ? new Date(+TWITTER_TIMESTAMP) : new Date(Date.now())
  
  console.log(`My HOPR address is ${hoprAddress}`)

  let bot: Bot
  switch (BOT_NAME) {
    case 'randobot':
      const { Randombot } = await import('./randobot')
      bot = new Randombot(hoprAddress, timestamp, twitterTimestamp)
      break
    case 'bouncerbot':
      const { Bouncebot } = await import('./bouncerbot')
      bot = new Bouncebot(hoprAddress, timestamp, twitterTimestamp)
      break
    case 'tweetbot':
      const { Tweetbot } = await import('./tweetbot')
      bot = new Tweetbot(hoprAddress, timestamp, twitterTimestamp)
      break
    case 'coverbot':
      const { Coverbot } = await import('./coverbot')
      bot = new Coverbot(hoprAddress, timestamp, twitterTimestamp)
  }
  await setupBot(bot, node)
}

start().catch((err) => {
  console.error('Fatal Error:', err)
  process.exit()
})
