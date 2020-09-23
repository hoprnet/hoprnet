import { BOT_NAME, BOT_TIMESTAMP, TWITTER_TIMESTAMP } from './utils/env'
import { setupBot, Bot } from './bots/bot'
import Core from './lib/hopr/core'

const start = async () => {
  const node = await new Core()
  await node.start()
  const hoprAddress = await node.address('hopr')

  const timestamp = BOT_TIMESTAMP ? new Date(+BOT_TIMESTAMP) : new Date(Date.now())
  const twitterTimestamp = TWITTER_TIMESTAMP ? new Date(+TWITTER_TIMESTAMP) : new Date(Date.now())

  console.log(`My HOPR address is ${hoprAddress}`)

  let bot: Bot
  switch (BOT_NAME) {
    case 'randobot':
      const { Randombot } = await import('./bots/randobot')
      bot = new Randombot(hoprAddress, timestamp, twitterTimestamp)
      break
    case 'bouncerbot':
      const { Bouncebot } = await import('./bots/bouncerbot')
      bot = new Bouncebot(hoprAddress, timestamp, twitterTimestamp)
      break
    case 'tweetbot':
      const { Tweetbot } = await import('./bots/tweetbot')
      bot = new Tweetbot(hoprAddress, timestamp, twitterTimestamp)
      break
    case 'coverbot':
      const { Coverbot } = await import('./bots/coverbot')
      bot = new Coverbot(hoprAddress, timestamp, twitterTimestamp)
  }
  await setupBot(bot, node)
}

start().catch((err) => {
  console.error('Fatal Error:', err)
  process.exit()
})
