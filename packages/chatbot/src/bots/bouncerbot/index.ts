import {getRandomItemFromList} from '../../utils/utils'
import {IMessage} from '../../message/message'
import {TweetMessage} from '../../lib/twitter/twitter'
import {Bot} from '../bot'
import {payDai} from '../../lib/linkdrop'
import response from './response.json'

const sendMessage = (from: string, message: IMessage, annonymous?: boolean) => {
  //@TODO Replace for actually sending message.
  console.log('Sending Message...')
}

enum NodeStates {
  IsHinted,
  RequiresProof,
  InGuestList,
}

export class Bouncebot implements Bot {
  botName: string
  address: string
  timestamp: Date
  twitterTimestamp: Date
  status: Map<string, NodeStates>
  messagesCounter: Map<string, number>
  winners: Map<string, boolean>
  winnersTwitter: Map<string, boolean>

  constructor(address: string, timestamp: Date, twitterTimestamp: Date) {
    this.address = address
    this.timestamp = timestamp
    this.twitterTimestamp = twitterTimestamp
    this.botName = '🥊 Bouncerbot'
    this.status = new Map<string, NodeStates>()
    this.messagesCounter = new Map<string, number>()
    this.winners = new Map<string, boolean>()
    this.winnersTwitter = new Map<string, boolean>()
    console.log(`${this.botName} has been added`)
  }

  async handleMessage(message: IMessage) {
    console.log(`${this.botName} <- ${message.from}: ${message.text}`)
    if (this.winners.get(message.from)) {
      return sendMessage(message.from, {
        from: this.address,
        text: response['alreadyWinner'],
      })
    }
    if (this.status.get(message.from) == NodeStates.RequiresProof) {
      try {
        await this.handleRequiresProof(message)
      } catch (err) {
        console.error(`Error while checking proof: ${err}`)
        sendMessage(message.from, {
          from: this.address,
          text: ` That doesn’t look like a tweet, err, proof! Try again with a valid one.`,
        })
      }
    } else if (message.text.match(/.*?\b(party)/i)) {
      if (this.status.has(message.from)) {
        switch (this.status.get(message.from)) {
          case NodeStates.IsHinted:
            this.handleIsHinted(message)
            break
          case NodeStates.InGuestList:
            this.handleGuest(message)
            break
        }
      } else this.handleNew(message)
    } else {
      sendMessage(message.from, {
        from: this.address,
        text: ` No loitering! I'm ${this.botName}, and if you want to get inside you'll need to tell me why you're here.`,
      })
    }
  }

  async handleRequiresProof(message) {
    const tweet = new TweetMessage(message.text)
    await tweet.fetch()
    if (this.winnersTwitter.get(tweet.screen_name)) {
      return sendMessage(message.from, {
        from: this.address,
        text: response['alreadyTwitterWinner'],
      })
    }
    if (tweet.isBlackListed(tweet.screen_name)) {
      return sendMessage(message.from, {
        from: this.address,
        text: response['blacklisted'],
      })
    }
    if (!tweet.isAfterTimestamp(this.twitterTimestamp)) {
      return sendMessage(message.from, {
        from: this.address,
        text: getRandomItemFromList(response['tweetIsOld']),
      })
    }
    if (!tweet.hasTag('hoprgames')) {
      return sendMessage(message.from, {
        from: this.address,
        text: getRandomItemFromList(response['tweetHasNoTag']),
      })
    }
    if (!tweet.hasMention('hoprnet')) {
      return sendMessage(message.from, {
        from: this.address,
        text: getRandomItemFromList(response['tweetHasNoMention']),
      })
    }
    if (!tweet.hasSameHOPRNode(message.from)) {
      return sendMessage(message.from, {
        from: this.address,
        text: getRandomItemFromList(response['tweetHasWrongNode']),
      })
    }
    sendMessage(message.from, {
      from: this.address,
      text: getRandomItemFromList(response['tweetSuccess']),
    })
    this.status.set(message.from, NodeStates.InGuestList)
    this.winnersTwitter.set(tweet.screen_name, true)
    setTimeout(this.welcomeUser.bind(this), 2000, message)
  }

  handleIsHinted(message) {
    sendMessage(message.from, {
      from: this.address,
      text: getRandomItemFromList(response['isHinted']),
    })
    this.status.set(message.from, NodeStates.RequiresProof)
  }

  handleGuest(message) {
    sendMessage(message.from, {
      from: this.address,
      text: getRandomItemFromList(response['isGuest']),
    })
  }

  handleNew(message) {
    sendMessage(message.from, {
      from: this.address,
      text: getRandomItemFromList(response['isNewUser']),
    })
    this.messagesCounter.set(message.from, (this.messagesCounter.get(message.from) || 0) + 1)
    console.log(`Messages from ${message.from} so far are ${this.messagesCounter.get(message.from)}`)
    if (this.messagesCounter.get(message.from) > 3) {
      setTimeout(this.hintUser.bind(this), 5000, message)
    }
  }

  async welcomeUser(message) {
    const payUrl = await payDai()
    console.log(`Payment link generated: ${payUrl}`)
    sendMessage(message.from, {
      from: this.address,
      text: response['guestWelcome'] + payUrl,
    })
    this.winners.set(message.from, true)
  }

  hintUser(message) {
    console.log(`Hinting ${message.from} about the party...`)
    sendMessage(
      message.from,
      {
        from: this.address,
        text: response['hint'],
      },
      true,
    )
    this.status.set(message.from, NodeStates.IsHinted)
  }
}
