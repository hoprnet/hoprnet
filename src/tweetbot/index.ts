import { getMessageStream, sendMessage } from '../utils'
import { ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'
import { Message } from '../message'
import { TWITTER_API_ACCESS_TOKEN, TWITTER_API_KEY, TWITTER_API_SECRET, TWITTER_API_ACCESS_TOKEN_SECRET } from '../env'
import TwitterClient from '@hoprnet/twitter-api-client';


const directory = {}
const winners = []

const twitterClient = new TwitterClient({
  apiKey: TWITTER_API_KEY,
  apiSecret: TWITTER_API_SECRET,
  accessToken: TWITTER_API_ACCESS_TOKEN,
  accessTokenSecret: TWITTER_API_ACCESS_TOKEN_SECRET,
});

enum STATUS {
  NEW_PARTICIPANT = 0,
  INTRODUCED = 1,
  RULES_GIVEN = 2,
  BOUNTY_COMPLETED = 3
}

enum MESSAGES {
  INTRO = `Hi! I’m TweetBot! Nice to meet you. How’re you doing?`,
  RULES = `First, send a tweet tagging @hoprnet which includes #HOPRGames and your HOPR node address. Then send me the URL. If you're one of the first 30 participants, I’ll get you some DAI!'`,
  NO_TWEET = 'Sorry... I couldn’t find a tweet in your message! Try again!',
  NO_HOPR_ACCOUNT = 'Hmm... that’s certainly a tweet, but I can’t see @hoprnet in it!',
  NO_HOPR_HASHTAG = 'Hey! That’s a neat tweet, but it doesn’t include the #HOPRGames tag!',
  NO_HOPR_ADDRESS = 'Good tweet! Don’t forget to include your HOPR node address though :)',
  NO_HOPR_ADDRESS_MISMATCH = 'Sorry! You can only send your tweet from a node you control. Nice try tho!',
  ALREADY_WINNER = 'You already won! Please don’t forget to fill in the form https://forms.gle/YZrrrBeT8r9qG78K6 to claim your reward',
  SUCCESS = `Congratulations! And thanks for supporting HOPR! Please fill our form https://forms.gle/YZrrrBeT8r9qG78K6 to get your reward.`,
  FAILURE = 'Hmm... something went wrong. Make sure you send me the full URL, including https.'
}

export default async (hoprAddress) => {
  const botName = '🐦 Tweetbot'
  console.log(`${botName} has been added`);

  const { client, stream } = await getMessageStream()

  stream
    .on('data', async (data) => {
      try {
        const [messageBuffer] = data.array
        const res = new ListenResponse()
        res.setPayload(messageBuffer)

        const message = new Message(res.getPayload_asU8()).toJson()
        console.log(`${botName} <- ${message.from}: ${message.text}`)

        let response;
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
          const [tweet] = message.text.match(/https:\/\/twitter.com.*?$/i)
          const tweetId = (tweet_regexed => tweet_regexed.pop())(tweet.split('/') || [])

          console.log(`${botName} <- ${message.from}: Obtained tweet with ID ${tweetId}`)

          const data = await twitterClient.tweets.statusesShowById({ id: tweetId })
          const { hashtags, user_mentions } = data.entities
          const tweetContent = data.text;

          console.log(`${botName} <- ${message.from}: Obtained tweet with Text ${tweetContent}`)
          console.log(`${botName} <- ${message.from}: Obtained tweet with Hashtags ${JSON.stringify(hashtags)}`)
          console.log(`${botName} <- ${message.from}: Obtained tweet with User Mentions ${JSON.stringify(user_mentions)}`)

          if (hashtags.some(hashtag => (hashtag.text as string).toLowerCase() === 'hoprgames')) {
            if(user_mentions.some(user => (user.screen_name as string).toLowerCase() === 'hoprnet')) {
              if(tweetContent.match(/16Uiu2HA.*?$/i)) {
                const [participantHOPRAddress_regexed] = tweetContent.match(/16Uiu2HA.*?$/i)
                const participantHOPRAddress = participantHOPRAddress_regexed.substr(0, 53)
                if(participantHOPRAddress === message.from) {
                  if(winners.includes(message.from)) {
                    response = MESSAGES.ALREADY_WINNER;
                  } else {
                    winners.push(message.from);
                    response = MESSAGES
                  }
                } else {
                  response = MESSAGES.NO_HOPR_ADDRESS_MISMATCH;
                }
              } else {
                response = MESSAGES.NO_HOPR_ADDRESS
              }
              response = MESSAGES.SUCCESS
            } else {
              console.log(`${botName} <- ${message.from}: No @hoprnet in Tweet ${tweetId}: ${JSON.stringify(user_mentions)}`)
              response = MESSAGES.NO_HOPR_ACCOUNT 
            }
          } else {
            console.log(`${botName} <- ${message.from}: No #HOPRgames in Tweet ${tweetId}: ${JSON.stringify(hashtags)}`)
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
          const [input] = message.text.match(/winners\ [0-9]?$/i);
          const [_, index] = input.split(' ');
          response = ~~index > winners.length ?
            `Winner #${index}: ${winners[~~index]}` :
            'Sorry, that winner doesn’t exist'
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
