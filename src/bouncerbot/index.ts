import { getMessageStream, sendMessage, getRandomItemFromList } from '../utils'
import { ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'
import { Message } from '../message'
import response from './response.json'


enum NodeStates {
  IsHinted,
  RequiresProof,
  InGuestList,
}

class Bouncebot {
  botName: string
  hoprAddress: string
  status: Map<string, NodeStates>

  constructor(hoprAddress: string) {
    this.hoprAddress = hoprAddress
    this.botName = '🥊 Bouncebot'
    this.status = new Map<string, NodeStates>()
    console.log(`${this.botName} has been added`)
  }

  async init() {
    const { client, stream } = await getMessageStream()
    stream
    .on('data', (data) => {
      try {
        const [messageBuffer] = data.array
        const res = new ListenResponse()
        res.setPayload(messageBuffer)
        const message = new Message(res.getPayload_asU8()).toJson()
        this.handleMessage(message)
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

  handleMessage(message) {
    console.log(`${this.botName} <- ${message.from}: ${message.text}`)

    if (message.text === 'Party') {
      if (this.status.has(message.from)) { 
        switch (this.status.get(message.from)) {
           case NodeStates.IsHinted:
             this.handleIsHinted(message)
             break;
           case NodeStates.RequiresProof:
             this.handleRequiresProof(message)
             break;
           case NodeStates.InGuestList:
             this.handleGuest(message)
             break;
         } 
      } else this.handleNew(message)
    } else {
      sendMessage(message.from, {
        from: this.hoprAddress,
        text: ` ${this.botName} isn’t amused...`,
      })
    }
  }

  handleRequiresProof(message) {
    let check = true
    // check if the the tweet is valid
    if (check) {
      sendMessage(message.from, {
        from: this.hoprAddress,
        text: getRandomItemFromList(response['tweetSuccess']),
      })
      this.status.set(message.from, NodeStates.InGuestList)
      setTimeout(this.welcomeUser.bind(this), 2000, message)
    } else {
      sendMessage(message.from, {
        from: this.hoprAddress,
        text: getRandomItemFromList(response['tweetFailure']),
      })
    }
  }

  handleIsHinted(message) {
    sendMessage(message.from, {
      from: this.hoprAddress,
      text: getRandomItemFromList(response['isHinted']),
    })
    this.status.set(message.from, NodeStates.RequiresProof)
  }

  handleGuest(message) {
    sendMessage(message.from, {
      from: this.hoprAddress,
      text: getRandomItemFromList(response['isGuest']),
    })  
  }

  handleNew(message) {
    sendMessage(message.from, {
      from: this.hoprAddress,
      text: getRandomItemFromList(response['isNewUser']),
    })
    setTimeout(this.hintUser.bind(this), 5000, message)
  }

  welcomeUser(message) {
    sendMessage(message.from, {
      from: this.hoprAddress,
      text: response['guestWelcome']
    })
  }

  hintUser(message) {
    sendMessage(message.from, {
        from: this.hoprAddress,
        text: response['hint'],
      })
    this.status.set(message.from, NodeStates.IsHinted)
  }

}

export default async (hoprAddress) => {
  const bot = new Bouncebot(hoprAddress)
  await bot.init()
}
