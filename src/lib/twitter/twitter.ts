import {
  TWITTER_API_ACCESS_TOKEN,
  TWITTER_API_KEY,
  TWITTER_API_SECRET,
  TWITTER_API_ACCESS_TOKEN_SECRET,
  TWITTER_BLACKLISTED,
} from '../../utils/env'
import TwitterClient from '@hoprnet/twitter-api-client'
import tweetMock from './tweetMock.json'
import { getHOPRNodeAddressFromContent } from '../../utils/utils'
import debug from 'debug'


const log = debug('hopr-chatbot:twitter')

const twitterClient = new TwitterClient({
  apiKey: TWITTER_API_KEY,
  apiSecret: TWITTER_API_SECRET,
  accessToken: TWITTER_API_ACCESS_TOKEN,
  accessTokenSecret: TWITTER_API_ACCESS_TOKEN_SECRET,
})

export class TweetState {
  hasMention: boolean = false
  hasTag: boolean = false
  sameNode: boolean = false

  public isValid() {
    return this.hasTag && this.hasMention && this.sameNode
  }

  public validateNode() {
      this.hasMention = true
      this.hasTag = true
      this.sameNode = true
  }
}

export class TweetMessage {
  url: string
  id: string
  status: TweetState
  created_at: Date
  screen_name: string
  hasfetched: boolean
  followers_count: number
  hashtags: any
  user_mentions: any
  content: string

  constructor(url: string) {
    const tweet = url.match(/https:\/\/twitter.com.*?$/i)
    if (!tweet) throw new Error('Invalid Tweet Url')
    this.id = ((tweet_regexed) => tweet_regexed.pop())(tweet[0].split('/') || [])
    this.url = url
    this.hasfetched = false
  }

  async fetch(options?: { mock: boolean }) {
    this.status = new TweetState()
    const data =
      options && options.mock
        ? tweetMock
        : await twitterClient.tweets.statusesShowById({ id: this.id, tweet_mode: 'extended' })
    this.url = `https://twitter.com/${data.user.screen_name}/status/${data.id_str}`
    this.id = `${data.id_str}`
    this.hashtags = data.entities.hashtags
    this.user_mentions = data.entities.user_mentions
    this.content = data.full_text || data.text
    this.followers_count = data.user.followers_count
    this.screen_name = data.user.screen_name
    this.created_at = new Date(data.created_at)
    this.hasfetched = true
  }

  isAfterTimestamp(timestamp: Date): boolean {
    return this.created_at > timestamp
  }

  hasTag(tag: string): boolean {
    return this.hashtags.some((hashtag) => (hashtag.text as string).toLowerCase() === tag)
  }

  hasMention(mention: string): boolean {
    return this.user_mentions.some((user) => (user.screen_name as string).toLowerCase() === mention)
  }

  isBlackListed(screen_name: string): boolean {
    const alreadyParticipants = TWITTER_BLACKLISTED.split(',')
    return alreadyParticipants.includes(screen_name)
  }

  hasEnoughFollowers(followers_count: number): boolean {
    //@TODO Move this to an env variable for later usage
    return followers_count > 100
  }

  getHOPRNode(options?: { mock: boolean, hoprNode: string}): string {
    log('- getHOPRNode | Starting to retrieve HOPR Node from Tweet')
    options && options.mock && log(`- getHOPRNode | Mock has been given, only reading ${options.hoprNode} now.`)
    return options.mock ? options.hoprNode : getHOPRNodeAddressFromContent(this.content)
  }

  validateTweetStatus() {
    return this.status.validateNode()
  }

  hasSameHOPRNode(hoprAddress: string): boolean {
    return this.content.match(/16Uiu2HA.*?$/is)
      ? ((tweetContent) => {
          const [participantHOPRAddress_regexed] = tweetContent.match(/16Uiu2HA.*?$/is)
          const participantHOPRAddress = participantHOPRAddress_regexed.substr(0, 53)
          return participantHOPRAddress === hoprAddress
        })(this.content)
      : false
  }
}
