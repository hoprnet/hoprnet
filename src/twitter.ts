import { TWITTER_API_ACCESS_TOKEN, TWITTER_API_KEY, TWITTER_API_SECRET, TWITTER_API_ACCESS_TOKEN_SECRET } from './env'
import TwitterClient from '@hoprnet/twitter-api-client'

const twitterClient = new TwitterClient({
  apiKey: TWITTER_API_KEY,
  apiSecret: TWITTER_API_SECRET,
  accessToken: TWITTER_API_ACCESS_TOKEN,
  accessTokenSecret: TWITTER_API_ACCESS_TOKEN_SECRET,
})


export class TweetMessage {
    url: string
    id: string
    hasfetched: boolean
    hashtags: any
    user_mentions: any
    content: string

    constructor(url: string) {
        const tweet = url.match(/https:\/\/twitter.com.*?$/i)
        if (!tweet) throw new Error('Invalid Tweet Url')
        this.id = (tweet_regexed => tweet_regexed.pop())(tweet[0].split('/') || [])
        this.url = url
        this.hasfetched = false
    }

    async fetch() {
        const data = await twitterClient.tweets.statusesShowById({ id: this.id })
        this.hashtags = data.entities.hashtags
        this.user_mentions = data.entities.user_mentions
        this.content = data.text
        this.hasfetched = true
        console.log('Obtained the following hashtags', this.hashtags);
        console.log('Obtained the following user_mentions', this.user_mentions);
        console.log('Obtained the following content', this.content);
    }

    hasTag(tag: string): boolean {
        return this.hashtags.some(hashtag => (hashtag.text as string).toLowerCase() === tag)
    }

    hasMention(mention: string): boolean {
        return this.user_mentions.some(user => (user.screen_name as string).toLowerCase() === mention)
    }
}
