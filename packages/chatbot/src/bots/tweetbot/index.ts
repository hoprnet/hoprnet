import { TweetMessage } from '../../lib/twitter/twitter'
import { Bot } from '../bot'
import { IMessage } from '../../message/message'

const directory = {}
const winners = []

enum STATUS {
  NEW_PARTICIPANT = 0,
  INTRODUCED = 1,
  RULES_GIVEN = 2,
  BOUNTY_COMPLETED = 3,
}

enum MESSAGES {
  INTRO = `Hi! I’m TweetBot! Nice to meet you. How’re you doing?`,
  RULES = `First, send a tweet tagging @hoprnet which includes #HOPRGames and your HOPR node address. Then send me the URL. If you're one of the first 30 successful bounty hunters, you'll get 10 DAI!'`,
  NO_TWEET = 'Sorry... I couldn’t find a tweet in your message! Try again!',
  NO_HOPR_ACCOUNT = 'Hmm... that’s certainly a tweet, but I can’t see @hoprnet in it!',
  NO_HOPR_HASHTAG = 'Hey! That’s a neat tweet, but it doesn’t include the #HOPRGames tag!',
  NO_HOPR_ADDRESS = 'Good tweet! Don’t forget to include your HOPR node address though :)',
  NO_HOPR_ADDRESS_MISMATCH = 'Sorry! You can only send your tweet from a node you control. Nice try tho!',
  ALREADY_WINNER = 'You already won! Please don’t forget to fill in the form https://forms.gle/YZrrrBeT8r9qG78K6 to claim your reward',
  SUCCESS = `Congratulations! And thanks for supporting HOPR! Please fill our form https://forms.gle/YZrrrBeT8r9qG78K6 to get your reward.`,
  FAILURE = 'Hmm... something went wrong. Make sure you send me the full URL, including https.',
}

export class Tweetbot implements Bot {
  botName: string
  address: string
  timestamp: Date
  twitterTimestamp: Date

  constructor(address: string, timestamp: Date, twitterTimestamp: Date) {
    this.address = address
    this.timestamp = timestamp
    this.twitterTimestamp = twitterTimestamp
    this.botName = '🐦 Tweetbot'
    console.log(`${this.botName} has been added`)
  }

  async handleMessage(message: IMessage) {
    console.log(`${this.botName} <- ${message.from}: ${message.text}`)
    let response
    /*
     * We do a few checks on the messages received by the user.
     * First time (i.e. STATUS.NEW_PARTICIPANT)
     *
     */
    if (!directory[message.from] || directory[message.from] === STATUS.NEW_PARTICIPANT) {
      directory[message.from] = STATUS.INTRODUCED
      response = MESSAGES.INTRO
    } else if (message.text.match(/rules?$/i) || directory[message.from] === STATUS.INTRODUCED) {
      directory[message.from] = STATUS.RULES_GIVEN
      response = MESSAGES.RULES
    } else if (message.text.match(/https:\/\/twitter.com.*?$/i)) {
      const tweet = new TweetMessage(message.text)
      await tweet.fetch()

      console.log(`${this.botName} <- ${message.from}: Obtained tweet with ID ${tweet.id}`)
      console.log(`${this.botName} <- ${message.from}: Obtained tweet with Text ${tweet.content}`)
      console.log(`${this.botName} <- ${message.from}: Obtained tweet with Hashtags ${JSON.stringify(tweet.hashtags)}`)
      console.log(
        `${this.botName} <- ${message.from}: Obtained tweet with User Mentions ${JSON.stringify(tweet.user_mentions)}`,
      )

      if (tweet.hasTag('hoprgames')) {
        if (tweet.hasMention('hoprnet')) {
          if (tweet.content.match(/16Uiu2HA.*?$/i)) {
            const [participantHOPRAddress_regexed] = tweet.content.match(/16Uiu2HA.*?$/i)
            const participantHOPRAddress = participantHOPRAddress_regexed.substr(0, 53)
            if (participantHOPRAddress === message.from) {
              if (winners.includes(message.from)) {
                response = MESSAGES.ALREADY_WINNER
              } else {
                winners.push(message.from)
                response = MESSAGES
              }
            } else {
              response = MESSAGES.NO_HOPR_ADDRESS_MISMATCH
            }
          } else {
            response = MESSAGES.NO_HOPR_ADDRESS
          }
          response = MESSAGES.SUCCESS
        } else {
          console.log(
            `${this.botName} <- ${message.from}: No @hoprnet in Tweet ${tweet.id}: ${JSON.stringify(
              tweet.user_mentions,
            )}`,
          )
          response = MESSAGES.NO_HOPR_ACCOUNT
        }
      } else {
        console.log(
          `${this.botName} <- ${message.from}: No #HOPRgames in Tweet ${tweet.id}: ${JSON.stringify(tweet.hashtags)}`,
        )
        response = MESSAGES.NO_HOPR_HASHTAG
      }
    } else {
      response = MESSAGES.FAILURE
    }

    /*
     * Some administrative commands to make the interaction with
     * our tweet bot a bit easier.
     */
    if (message.text.match(/winners?$/i)) {
      response = `So far we’ve had ${winners.length} winners.`
    }
    if (message.text.match(/winners\ [0-9]?$/i)) {
      const [input] = message.text.match(/winners\ [0-9]?$/i)
      const [_, index] = input.split(' ')
      response = ~~index > winners.length ? `Winner #${index}: ${winners[~~index]}` : 'Sorry, that winner doesn’t exist'
    }

    // @TODO Actually send message
    // sendMessage(message.from, {
    //   from: this.address,
    //   text: ` ${response}`,
    // })
  }
}
