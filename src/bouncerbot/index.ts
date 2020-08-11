import { sendMessage, getRandomItemFromList } from '../utils'
import { IMessage } from '../message'
import { ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'
import { TweetMessage } from '../twitter'
import { Bot } from '../bot'
import response from './response.json'


enum NodeStates {
  IsHinted,
  RequiresProof,
  InGuestList,
}

export class Bouncebot implements Bot{
  botName: string
  address: string
  status: Map<string, NodeStates>

  constructor(address: string) {
    this.address = address
    this.botName = '🥊 Bouncebot'
    this.status = new Map<string, NodeStates>()
    console.log(`${this.botName} has been added`)
  }

  handleMessage(message: IMessage) {
    console.log(`${this.botName} <- ${message.from}: ${message.text}`)

    if (this.status.get(message.from) == NodeStates.RequiresProof)
      this.handleRequiresProof(message)
    else if (message.text === 'Party') {
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
        text: ` ${this.botName} isn’t amused...`,
      })
    }
  }

  async handleRequiresProof(message) {
    const tweet = new TweetMessage(message.text)
    await tweet.fetch()
    // check if the the tweet is valid
    if (tweet.hashtags('hoprgames')) {
      sendMessage(message.from, {
        from: this.address,
        text: getRandomItemFromList(response['tweetSuccess']),
      })
      this.status.set(message.from, NodeStates.InGuestList)
      setTimeout(this.welcomeUser.bind(this), 2000, message)
    } else {
      sendMessage(message.from, {
        from: this.address,
        text: getRandomItemFromList(response['tweetFailure']),
      })
    }
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
    setTimeout(this.hintUser.bind(this), 5000, message)
  }

  welcomeUser(message) {
    sendMessage(message.from, {
      from: this.address,
      text: response['guestWelcome']
    })
  }

  hintUser(message) {
    sendMessage(message.from, {
        from: this.address,
        text: response['hint'],
      })
    this.status.set(message.from, NodeStates.IsHinted)
  }
}
