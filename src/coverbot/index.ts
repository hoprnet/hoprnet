import { sendMessage } from '../utils'
import Web3 from 'web3';
import { Bot } from '../bot'
import { IMessage } from '../message'
import { TweetMessage, TweetState } from '../twitter'
//@TODO: Isolate these utilities to avoid importing the entire package
import { convertPubKeyFromB58String, u8aToHex } from '@hoprnet/hopr-utils'
import { Utils } from '@hoprnet/hopr-core-ethereum'
import fs from 'fs'


//@TODO: Move this to an environment variable or read from a contract
const XDAI_THRESHOLD = 0.001
const VERIFICATION_CYCLE_IN_MS = 5000

type HoprNode = {
  id: string,
  tweetId: string,
  tweetUrl: string
}

enum NodeStates {
  newUnverifiedNode = 'UNVERIFIED',
  tweetVerificationFailed = 'FAILED_TWITTER_VERIFICATION',
  tweetVerificationSucceeded = 'SUCCEEDED_TWITTER_VERIFICATION',
  xdaiBalanceFailed = 'FAILED_XDAI_BALANCE_VERIFICATION',
  xdaiBalanceSucceeded = 'SUCCEEDED_XDAI_BALANCE_VERIFICATION',
  verifiedNode = 'VERIFIED'
}

enum BotCommands {
  rules,
  status,
  verify
}

const BotResponses = {
  [BotCommands.rules]: `\n
    Welcome to the xHOPR incentivized network.

    1. Post a tweet with your HOPR Address and the tag #HOPRNetwork
    2. Load ${XDAI_THRESHOLD} xDAI into your HOPR Ethereum Address
    3. Send me the link to your tweet (don‘t delete it!)
    4. Keep your tweet and node alive, and I'll slowly send xHOPR to you.
    
    For more information, go to https://saentis.hoprnet.org
  `,
  [BotCommands.status]: (status: NodeStates) => `\n
    Your current status is: ${status}
  `,
  [BotCommands.verify]: `\n
    Verifying if your node is still up...
  `
}

const NodeStateResponses = {
  [NodeStates.newUnverifiedNode]: BotResponses[BotCommands.rules],
  [NodeStates.tweetVerificationFailed]: (tweetStatus: TweetState) => `\n
    Your tweet has failed the verification. Please make sure to follow the rules.

    Here is the current status of your tweet:
    1. Tagged @hoprnet: ${tweetStatus.hasMention}
    2. Used #HOPRNetwork: ${tweetStatus.hasTag}
    3. Includes your node: ${tweetStatus.sameNode}

    Please try again with a different tweet.
  `,
  [NodeStates.tweetVerificationSucceeded]: `\n
    Your tweet has succeeded verification. Please do no delete this tweet, as we will
    use it multiple times to verify and connect to your node.

    We’ll now proceed to check that your HOPR Ethereum address has at least ${XDAI_THRESHOLD} xDAI.
    If you need xDAI, you always swap DAI to xDAI using https://dai-bridge.poa.network/.
  `,
  [NodeStates.xdaiBalanceFailed]: (xDaiBalance: number) => `\n
    Your node does not have at least ${XDAI_THRESHOLD} xDAI. Currently, your node has ${xDaiBalance} xDAI.

    To participate in our incentivized network, please make sure to add the missing amount of xDAI.
  `,
  [NodeStates.xdaiBalanceSucceeded]: (xDaiBalance: number) => `\n
    Your node has ${xDaiBalance} xDAI. You are ready to go!
    In short, our bot will open a payment channel to your node.

    Please keep your balance, tweet and node running.
    Only doing so, you can relay packets and get tickets.
    Don't forget to redeem them every once in a while!

    For more information, go to https://saentis.hoprnet.org

    Thank you for participating in our incentivized network!
  `,
  [NodeStates.verifiedNode]: `\n
    Congratulations! I’ll shortly use you as a cover traffic node
    and pay you in xHOPR tokens for your service.
  `
}

export class Coverbot implements Bot {
  botName: string
  address: string
  timestamp: Date
  status: Map<string, NodeStates>
  tweets: Map<string, TweetMessage>
  twitterTimestamp: Date

  verifiedHoprNodes: Map<string, HoprNode>

  verificationTimeout: NodeJS.Timeout;
  xdaiWeb3: Web3;
  ethereumAddress: string;

  constructor(address: string, timestamp: Date, twitterTimestamp: Date) {
    this.address = address
    this.timestamp = timestamp
    this.status = new Map<string, NodeStates>()
    this.tweets = new Map<string, TweetMessage>()
    this.twitterTimestamp = twitterTimestamp
    this.botName = '💰 Coverbot'
    console.log(`${this.botName} has been added`)

    this.ethereumAddress = null;
    this.xdaiWeb3 = new Web3(new Web3.providers.HttpProvider('https://dai.poa.network'));
    this.verificationTimeout = setInterval(this._verificationCycle.bind(this), VERIFICATION_CYCLE_IN_MS)

    this.verifiedHoprNodes = new Map<string, HoprNode>()
  }

  protected async dumpData() {
    //@TODO: Ideally we move this to a more suitable place.
    if(!this.ethereumAddress) {
      const pubkey = await convertPubKeyFromB58String(this.address)
      const ethereumAddress = u8aToHex(await Utils.pubKeyToAccountId(pubkey.marshal()))
      this.ethereumAddress = ethereumAddress;
    }

    console.log('Checking dump...', Array.from(this.verifiedHoprNodes.values()))

    const state = {
      address: this.address,
      balance: await this.xdaiWeb3.eth.getBalance(this.ethereumAddress),
      available: 0,
      locked: 0,
      claimed: 0,
      connected: Array.from(this.verifiedHoprNodes.values()),
      refreshed: new Date().toISOString()
    }

    console.log('Storing nodes...')
    let pth = process.env.STATS_FILE
    fs.writeFileSync(pth, JSON.stringify(state), 'utf8')
    console.log("Stored", pth)
  }

  protected async _sendMessageOpeningChannels(recipient, message, intermediatePeers) {
    return sendMessage(recipient, {
      from: this.address,
      text: message,
    }, true, intermediatePeers)
  }

  protected async _verificationCycle() {
    console.log(`${VERIFICATION_CYCLE_IN_MS}ms has passed. Verifying nodes...`)

    this.dumpData()
    const _verifiedNodes = Array.from(this.verifiedHoprNodes.values());
    console.log('Verified nodes', _verifiedNodes);
    const randomIndex = Math.floor(Math.random() * _verifiedNodes.length);
    console.log('Random index', randomIndex);
    const hoprNode: HoprNode = _verifiedNodes[randomIndex]
    console.log('Trying to verify:', hoprNode);
    if (!hoprNode) {
      return;
    }

    try {
      const tweet = new TweetMessage(hoprNode.tweetUrl)
      await tweet.fetch()
      const _hoprNodeAddress = tweet.getHOPRNode()
      console.log('HoprNode Address', _hoprNodeAddress);
      if (_hoprNodeAddress.length === 0) {
        // We got no HOPR Node here.
        this.verifiedHoprNodes.delete(hoprNode.id)
      } else {
        this._sendMessageFromBot(_hoprNodeAddress, BotResponses[BotCommands.verify])
        //@TODO: We need to actually be able to get a message back from the user instead of waiting.
        setTimeout(async () => {
          this._sendMessageFromBot(_hoprNodeAddress, NodeStateResponses[NodeStates.verifiedNode])
          this._sendMessageOpeningChannels(this.address, `Packet relayed by ${_hoprNodeAddress}`, [_hoprNodeAddress])
        }, VERIFICATION_CYCLE_IN_MS/2)
      }
    } catch (err) {
      console.log('Err:', err);
      // Something failed. We better remove node.
      this.verifiedHoprNodes.delete(hoprNode.id)
    }
    console.log("Checking logs", Array.from(this.verifiedHoprNodes.values()));
  }

  protected _sendMessageFromBot(recipient, message) {
    return sendMessage(recipient, {
      from: this.address,
      text: message,
    })
  }

  protected async _verifyBalance(message: IMessage): Promise<[number, NodeStates]> {
    const pubkey = await convertPubKeyFromB58String(message.from)
    const nodeEthereumAddress = u8aToHex(await Utils.pubKeyToAccountId(pubkey.marshal()))
    //@TODO: Move this from hardcoded POA network to ENV_PROVIDER
    const weiBalance = await this.xdaiWeb3.eth.getBalance(nodeEthereumAddress)
    const balance = +Web3.utils.fromWei(weiBalance)

    return balance >= XDAI_THRESHOLD ? [balance, NodeStates.xdaiBalanceSucceeded] : [balance, NodeStates.xdaiBalanceFailed]
  }

  protected async _verifyTweet(message: IMessage): Promise<[TweetMessage, NodeStates]> {
    const tweet = new TweetMessage(message.text)
    this.tweets.set(message.from, tweet)

    //@TODO: Remove mock for production to ensure we process tweets.
    /*
    * Careful, it seems that the twitter API truncates some of the text
    * content, so if something isn't in the first 100 characters, it might
    * be left out of the parser.
    */
    await tweet.fetch({ mock: true })

    if (tweet.hasTag('hoprnetwork')) {
      tweet.status.hasTag = true
    }
    if(tweet.hasMention('hoprnet')) {
      tweet.status.hasMention = true
    }
    if(tweet.hasSameHOPRNode(message.from)) {
      tweet.status.sameNode = true
    }
    return tweet.status.isValid() ? [tweet, NodeStates.tweetVerificationSucceeded] : [tweet, NodeStates.tweetVerificationFailed]
  }

  async handleMessage(message: IMessage) {
    console.log(`${this.botName} <- ${message.from}: ${message.text}`)
    const [tweet, nodeState] = message.text.match(/https:\/\/twitter.com.*?$/i) ?
      await this._verifyTweet(message) :
      [undefined, NodeStates.newUnverifiedNode];

    switch(nodeState) {
      case NodeStates.newUnverifiedNode:
        this._sendMessageFromBot(message.from, NodeStateResponses[nodeState])
        break;
      case NodeStates.tweetVerificationFailed:
        this._sendMessageFromBot(message.from, NodeStateResponses[nodeState](this.tweets.get(message.from).status))
        break;
      case NodeStates.tweetVerificationSucceeded:
        this._sendMessageFromBot(message.from, NodeStateResponses[nodeState])
        const [balance, xDaiBalanceNodeState] = await this._verifyBalance(message)
        switch(xDaiBalanceNodeState) {
          case NodeStates.xdaiBalanceFailed:
            this._sendMessageFromBot(message.from, NodeStateResponses[xDaiBalanceNodeState](balance))
            break;
          case NodeStates.xdaiBalanceSucceeded:
            //@TODO Add this to a persistent store
            this.verifiedHoprNodes.set(message.from, {id: message.from, tweetId: tweet.id, tweetUrl: tweet.url })
            this._sendMessageFromBot(message.from, NodeStateResponses[xDaiBalanceNodeState](balance))
            break;
        }
        this._sendMessageFromBot(message.from, BotResponses[BotCommands.status](xDaiBalanceNodeState))
        break;
    }
    this._sendMessageFromBot(message.from, BotResponses[BotCommands.status](nodeState))
  }
}
