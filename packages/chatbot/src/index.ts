import { BOT_NAME, BOT_TIMESTAMP, TWITTER_TIMESTAMP } from './utils/env'
import { setupBot, Bot } from './bots/bot'
import Core from './lib/hopr/core'
import debug from 'debug'
import Web3 from 'web3'

const { fromWei } = Web3.utils
const log = debug('hopr-chatbot:main')
const error = debug('hopr-chatbot:main:error')

const main = async () => {
  log(`- main | Starting HOPR Core`)

  const node = await new Core()
  await node.start()
  const hoprAddress = await node.address('hopr')
  const nativeAddress = await node.address('native')
  const hoprBalance = fromWei(await node.getHoprBalance())
  const balance = fromWei(await node.getBalance())

  const timestamp = BOT_TIMESTAMP ? new Date(+BOT_TIMESTAMP) : new Date(Date.now())
  const twitterTimestamp = TWITTER_TIMESTAMP ? new Date(+TWITTER_TIMESTAMP) : new Date(Date.now())

  log(`- main | HOPR address: ${hoprAddress}`)

  let bot: Bot

  log(`- main | Creating Bot: ${BOT_NAME}`)
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
      bot = new Coverbot({ node, balance, hoprBalance }, nativeAddress, hoprAddress, timestamp, twitterTimestamp)
  }
  log(`- main | Bot Created: ${bot.botName}`)
  log(`- main | Setting up Bot on Node`)
  await setupBot(bot, node)
}

main().catch((err) => {
  error('- main | Exception:', err)
  process.exit()
})
