import { sendMessage, getHoprBalance, getHOPRNodeAddressFromContent, getStatus, sendXHOPR } from '../utils'
import Web3 from 'web3';
import { Bot } from '../bot'
import { IMessage } from '../message'
import { TweetMessage, TweetState } from '../twitter'
//@TODO: Isolate these utilities to avoid importing the entire package
import { convertPubKeyFromB58String, u8aToHex } from '@hoprnet/hopr-utils'
import { Utils } from '@hoprnet/hopr-core-ethereum'
import fs from 'fs'
import { Networks, HOPR_CHANNELS } from '@hoprnet/hopr-core-ethereum/lib/ethereum/addresses';
import { COVERBOT_DEBUG_MODE, COVERBOT_CHAIN_PROVIDER, COVERBOT_VERIFICATION_CYCLE_IN_MS, COVERBOT_XDAI_THRESHOLD } from '../env'


const { fromWei } = Web3.utils;
const RELAY_VERIFICATION_CYCLE_IN_MS = COVERBOT_VERIFICATION_CYCLE_IN_MS/2

type HoprNode = {
  id: string,
  address: string,
  tweetId: string,
  tweetUrl: string
}

enum NodeStates {
  newUnverifiedNode = 'UNVERIFIED',
  tweetVerificationFailed = 'FAILED_TWITTER_VERIFICATION',
  tweetVerificationSucceeded = 'SUCCEEDED_TWITTER_VERIFICATION',
  xdaiBalanceFailed = 'FAILED_XDAI_BALANCE_VERIFICATION',
  xdaiBalanceSucceeded = 'SUCCEEDED_XDAI_BALANCE_VERIFICATION',
  relayingNodeFailed = 'FAILED_RELAYING_PACKET',
  relayingNodeInProgress = 'IN_PROGRESS_RELAYING_PACKET',
  relayingNodeSucceded = 'SUCCEEDED_RELAYING_PACKET',
  onlineNode = 'ONLINE',
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
    2. Load ${COVERBOT_XDAI_THRESHOLD} xDAI into your HOPR Ethereum Address
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

    We’ll now proceed to check that your HOPR Ethereum address has at least ${COVERBOT_XDAI_THRESHOLD} xDAI.
    If you need xDAI, you always swap DAI to xDAI using https://dai-bridge.poa.network/.
  `,
  [NodeStates.xdaiBalanceFailed]: (xDaiBalance: number) => `\n
    Your node does not have at least ${COVERBOT_XDAI_THRESHOLD} xDAI. Currently, your node has ${xDaiBalance} xDAI.

    To participate in our incentivized network, please make sure to add the missing amount of xDAI.
  `,
  [NodeStates.xdaiBalanceSucceeded]: (xDaiBalance: number) => `\n
    Your node has ${xDaiBalance} xDAI. You are ready to go!
    In short, our bot will open a payment channel to your node.

    Please keep your balance, tweet and node running.

    For more information, go to https://saentis.hoprnet.org

    Thank you for participating in our incentivized network!
  `,
  [NodeStates.onlineNode]: `\n
    Node Online! Relaying a message to verify your ability to
    send messages to other nodes in the network.
  `,
  [NodeStates.verifiedNode]: `\n
    Verification Successful! I’ll shortly use you as a cover traffic node
    and pay you in xHOPR tokens for your service.

    For more information, go to https://saentis.hoprnet.org

    Thank you for participating in our incentivized network!
  `,
  [NodeStates.relayingNodeFailed]: `\n
    Relaying failed. We can reach you, but you can’t reach us...
    This could mean other errors though, so you always try again.

    For more information, go to https://saentis.hoprnet.org

    Drop by our Telegram (https://t.me/hoprnet) for any questions.
  `,
  [NodeStates.relayingNodeInProgress]: `\n
    Relaying in progress. We have received your message and are now
    waiting for a packet we are trying to relay on you. Please wait
    until we receive your packet or ${RELAY_VERIFICATION_CYCLE_IN_MS/1000}seconds
    pass out. Thank you for your patience!
  `,
  [NodeStates.relayingNodeSucceded]: `\n
    Relaying successful! We have obtained a packet annonymously from you,
    and are ready to go through the next verification process.
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
  relayTimeouts: Map<string, NodeJS.Timeout>
  verificationTimeout: NodeJS.Timeout

  xdaiWeb3: Web3;
  ethereumAddress: string;
  chainId: number;
  network: Networks;

  constructor(address: string, timestamp: Date, twitterTimestamp: Date) {
    this.address = address
    this.timestamp = timestamp
    this.status = new Map<string, NodeStates>()
    this.tweets = new Map<string, TweetMessage>()
    this.twitterTimestamp = twitterTimestamp
    this.botName = '💰 Coverbot'
    console.log(`${this.botName} has been added`)

    console.log(`⚡️ Network: ${COVERBOT_CHAIN_PROVIDER}`)
    console.log(`💸 Threshold: ${COVERBOT_XDAI_THRESHOLD}`)
    console.log(`🐛 Debug Mode: ${COVERBOT_DEBUG_MODE}`)
    console.log(`👀 Verification Cycle: ${COVERBOT_VERIFICATION_CYCLE_IN_MS}`)
    console.log(`🔍 Relaying Cycle: ${RELAY_VERIFICATION_CYCLE_IN_MS}`)

    this.ethereumAddress = null;
    this.chainId = null;
    this.network = null;

    this.xdaiWeb3 = new Web3(new Web3.providers.HttpProvider(COVERBOT_CHAIN_PROVIDER));
    this.verificationTimeout = setInterval(this._verificationCycle.bind(this), COVERBOT_VERIFICATION_CYCLE_IN_MS)

    this.verifiedHoprNodes = new Map<string, HoprNode>()
    this.relayTimeouts = new Map<string, NodeJS.Timeout>()

    console.log('📦 Setting up “database”')
    this.dumpData()
  }

  private async _getEthereumAddressFromHOPRAddress(hoprAddress: string): Promise<string> {
    const pubkey = await convertPubKeyFromB58String(hoprAddress)
    const ethereumAddress = u8aToHex(await Utils.pubKeyToAccountId(pubkey.marshal()))
    return ethereumAddress;
  }

  protected async dumpData() {
    //@TODO: Ideally we move this to a more suitable place.
    if(!this.ethereumAddress) {
      this.chainId = await Utils.getChainId(this.xdaiWeb3)
      this.network = Utils.getNetworkName(this.chainId) as Networks
      this.ethereumAddress = await this._getEthereumAddressFromHOPRAddress(this.address)
    }

    const connectedNodes = await getStatus()

    const state = {
      connectedNodes,
      env: {
        COVERBOT_CHAIN_PROVIDER,
        COVERBOT_DEBUG_MODE,
        COVERBOT_VERIFICATION_CYCLE_IN_MS,
        COVERBOT_XDAI_THRESHOLD
      },
      hoprCoverbotAddress: await this._getEthereumAddressFromHOPRAddress(this.address),
      hoprChannelContract: HOPR_CHANNELS[this.network],
      address: this.address,
      balance: fromWei(await this.xdaiWeb3.eth.getBalance(this.ethereumAddress)),
      available: fromWei(await getHoprBalance()),
      locked: 0, //@TODO: Retrieve balances from open channels.
      connected: Array.from(this.verifiedHoprNodes.values()),
      refreshed: new Date().toISOString()
    }

    let pth = process.env.STATS_FILE
    fs.writeFileSync(pth, JSON.stringify(state), 'utf8')
  }

  protected async _sendMessageOpeningChannels(recipient, message, intermediatePeers) {
    return sendMessage(recipient, {
      from: this.address,
      text: message,
    }, false, intermediatePeers)
  }

  protected async _verificationCycle() {
    console.log(`${COVERBOT_VERIFICATION_CYCLE_IN_MS}ms has passed. Verifying nodes...`)

    this.dumpData()

    const _verifiedNodes = Array.from(this.verifiedHoprNodes.values());
    const randomIndex = Math.floor(Math.random() * _verifiedNodes.length);
    const hoprNode: HoprNode = _verifiedNodes[randomIndex]

    if (!hoprNode) {
      console.log('No node found. Skipping...')
      return;
    }

    if (this.relayTimeouts.get(hoprNode.id)) {
      console.log('Node selected is going through relaying. Skipping...')
      return;
    }

    try {
      const tweet = new TweetMessage(hoprNode.tweetUrl)
      await tweet.fetch({ mock: COVERBOT_DEBUG_MODE })
      const _hoprNodeAddress = tweet.getHOPRNode()
      if (_hoprNodeAddress.length === 0) {
        // We got no HOPR Node here. Remove and update.
        this.verifiedHoprNodes.delete(hoprNode.id)
        this.dumpData()
      } else {
        this._sendMessageFromBot(_hoprNodeAddress, BotResponses[BotCommands.verify])
        /*
        * We switched from “send and forget” to “send and listen”
        * 1. We inmediately send a message to user, telling them we find them online.
        * 2. We use them as a relayer, expecting to get our message later.
        * 3. We save a timeout, to fail the node if the relayed package doesnt come back.
        * 4. We wait RELAY_VERIFICATION_CYCLE_IN_MS seconds for the relay to get back.
        *    4.1 If we don't get the message back before RELAY_VERIFICATION_CYCLE_IN_MS,
        *        then we remove the node from the verification table and redump data.
        *    4.2 If we DO get the message back, then we go and execute the payout function.
        */

        // 1.
        console.log(`Relaying node ${_hoprNodeAddress}, checking in ${RELAY_VERIFICATION_CYCLE_IN_MS}`)
        this._sendMessageFromBot(_hoprNodeAddress, NodeStateResponses[NodeStates.onlineNode])

        // 2.
        this._sendMessageOpeningChannels(this.address, ` Packet relayed by ${_hoprNodeAddress}`, [_hoprNodeAddress])

        // 3.
        this.relayTimeouts.set(_hoprNodeAddress, setTimeout(() => {
          // 4.1
          /*
          * The timeout passed, and we didn‘t get the message back.
          * 4.1.1 Internally log that this is the case.
          * 4.1.2 Let the node that we couldn't get our response back in time.
          * 4.1.3 Remove from timeout so they can try again somehow.
          * 4.1.4 Remove from our verified node and write to the stats.json
          */

          // 4.1.1
          console.log(`No response from ${_hoprNodeAddress}. Removing as valid node.`)

          // 4.1.2
          this._sendMessageFromBot(_hoprNodeAddress, NodeStateResponses[NodeStates.relayingNodeFailed])

          // 4.1.3
          this.relayTimeouts.delete(_hoprNodeAddress)
          this.verifiedHoprNodes.delete(_hoprNodeAddress)

          // 4.1.4
          this.dumpData()
        }, RELAY_VERIFICATION_CYCLE_IN_MS))
      }
    } catch (err) {
      console.log('Err:', err);
      // Something failed. We better remove node and update.
      this.verifiedHoprNodes.delete(hoprNode.id)
      this.dumpData()
    }
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
    const weiBalance = await this.xdaiWeb3.eth.getBalance(nodeEthereumAddress)
    const balance = +Web3.utils.fromWei(weiBalance)

    return balance >= COVERBOT_XDAI_THRESHOLD ? [balance, NodeStates.xdaiBalanceSucceeded] : [balance, NodeStates.xdaiBalanceFailed]
  }

  protected async _verifyTweet(message: IMessage): Promise<[TweetMessage, NodeStates]> {
    //@TODO: Catch error here.
    const tweet = new TweetMessage(message.text)
    this.tweets.set(message.from, tweet)

    await tweet.fetch({ mock: COVERBOT_DEBUG_MODE })

    if (tweet.hasTag('hoprnetwork')) {
      tweet.status.hasTag = true
    }
    if(tweet.hasMention('hoprnet')) {
      tweet.status.hasMention = true
    }
    if(tweet.hasSameHOPRNode(message.from) || COVERBOT_DEBUG_MODE ) {
      tweet.status.sameNode = true
    }
    return tweet.status.isValid() ? [tweet, NodeStates.tweetVerificationSucceeded] : [tweet, NodeStates.tweetVerificationFailed]
  }

  async handleMessage(message: IMessage) {
    console.log(`${this.botName} <- ${message.from}: ${message.text}`)

    if(message.from === this.address) {
      /*
      * We have done a succeful round trip!
      * 1. Lets avoid sending more messages to eternally loop
      *    messages across the network by returning within this if.
      * 2. Let's notify the user about the successful relay.
      * 3. Let's recover the timeout from our relayerTimeout
      *    and clear it before it removes the node.
      * 4. Let's pay the good node some sweet xHOPR for being alive
      *    and relaying messages successfully.
      */
      const relayerAddress = getHOPRNodeAddressFromContent(message.text)

      // 2.
      console.log(`Successful Relay: ${relayerAddress}`)
      this._sendMessageFromBot(relayerAddress, NodeStateResponses[NodeStates.relayingNodeSucceded])

      // 3.
      const relayerTimeout = this.relayTimeouts.get(relayerAddress)
      clearTimeout(relayerTimeout)
      this.relayTimeouts.delete(relayerAddress)

      // 4.
      await sendXHOPR(await this._getEthereumAddressFromHOPRAddress(relayerAddress), 1000000000000000)
      this._sendMessageFromBot(relayerAddress, NodeStateResponses[NodeStates.verifiedNode])

      // 1.
      return;
    }

    if(this.relayTimeouts.get(message.from)) {
      /*
      * There‘s a particular case where someone can send us a message while
      * we are trying to relay them a package. We'll skip the entire process
      * if this is the case, as the timeout will clear them out.
      *
      * 1. Detect if we have someone waiting for timeout (this if).
      * 2. If so, then quickly return them a message telling we are waiting.
      * 3. Return as to avoid going through the entire process again.
      *
      */

      // 2.
      this._sendMessageFromBot(message.from, NodeStateResponses[NodeStates.relayingNodeInProgress])

      // 3.
      return;
    }

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
            this.verifiedHoprNodes.set(message.from, {
              id: message.from,
              tweetId: tweet.id,
              tweetUrl: tweet.url,
              address: await this._getEthereumAddressFromHOPRAddress(message.from)
            })
            this._sendMessageFromBot(message.from, NodeStateResponses[xDaiBalanceNodeState](balance))
            break;
        }
        this._sendMessageFromBot(message.from, BotResponses[BotCommands.status](xDaiBalanceNodeState))
        break;
    }
    this._sendMessageFromBot(message.from, BotResponses[BotCommands.status](nodeState))
  }
}
