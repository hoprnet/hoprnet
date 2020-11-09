import Web3 from 'web3'
import { Bot } from '../bot'
import { IMessage } from '../../message/message'
import { TweetMessage } from '../../lib/twitter/twitter'
import FirebaseDatabase from '../../lib/firebase/api'
//@TODO: Isolate these utilities to avoid importing the entire package
import { convertPubKeyFromB58String, u8aToHex } from '@hoprnet/hopr-utils'
import { Utils } from '@hoprnet/hopr-core-ethereum'
import type { Network } from '@hoprnet/hopr-ethereum'
import {
  COVERBOT_DEBUG_MODE,
  COVERBOT_CHAIN_PROVIDER,
  COVERBOT_VERIFICATION_CYCLE_IN_MS,
  COVERBOT_XDAI_THRESHOLD,
  COVERBOT_VERIFICATION_CYCLE_EXECUTE,
  HOPR_ENVIRONMENT,
  FIREBASE_DATABASE_URL,
} from '../../utils/env'
import { NodeStates, VerifyTweetStates } from './types/states'
import { RELAY_VERIFICATION_CYCLE_IN_MS } from './utils/constants'
import { BalancedHoprNode, HoprNode } from './types/coverbot'
import Core from '../../lib/hopr/core'
import Instruction from './classes/instruction'
import { BotCommands } from './types/commands'
import { verificatorReducer } from './reducers/verificatorReducer'
import debug from 'debug'
import { coverTrafficCycle } from './cycles/coverTrafficCycle'
import { statsReducer } from './reducers/statsReducer'
import { adminReducer } from './reducers/adminReducer'
import { rulesReducer } from './reducers/rulesReducer'
import { loadData, dumpData } from './data/data'
import { helpReducer } from './reducers/helpReducer'

const log = debug('hopr-chatbot:coverbot')
const error = debug('hopr-chatbot:coverbot:error')

export class Coverbot implements Bot {
  node: Core
  initialBalance: string
  initialHoprBalance: string
  botName: string
  nativeAddress: string
  address: string
  timestamp: Date
  status: Map<string, NodeStates>
  tweets: Map<string, TweetMessage>
  twitterTimestamp: Date
  database: FirebaseDatabase

  verifiedHoprNodes: Map<string, HoprNode>
  relayTimeouts: Map<string, NodeJS.Timeout>
  verificationTimeout: NodeJS.Timeout

  xdaiWeb3: Web3
  ethereumAddress: string
  chainId: number
  network: Network
  loadedDb: boolean

  constructor(
    { node, hoprBalance, balance }: BalancedHoprNode,
    nativeAddress: string,
    address: string,
    timestamp: Date,
    twitterTimestamp: Date,
  ) {
    this.node = node
    this.initialBalance = balance
    this.initialHoprBalance = hoprBalance
    this.address = address
    this.nativeAddress = nativeAddress
    this.timestamp = timestamp
    this.status = new Map<string, NodeStates>()
    this.tweets = new Map<string, TweetMessage>()
    this.twitterTimestamp = twitterTimestamp
    this.botName = '💰 Coverbot'
    this.loadedDb = false
    this.database = new FirebaseDatabase({ databaseUrl: FIREBASE_DATABASE_URL })

    log(`- constructor | ${this.botName} has been added`)
    log(`- constructor | 🏠 HOPR Address: ${this.address}`)
    log(`- constructor | 🏡 Native Address: ${this.nativeAddress}`)
    log(`- constructor | ⛓ EVM Network: ${COVERBOT_CHAIN_PROVIDER}`)
    log(`- constructor | 📦 Firebase Database URL: ${FIREBASE_DATABASE_URL}`)
    log(`- constructor | 📦 Root Schema for Firebase: ${HOPR_ENVIRONMENT}`)
    log(`- constructor | 💸 Threshold: ${COVERBOT_XDAI_THRESHOLD}`)
    log(`- constructor | 💰 Native Balance: ${this.initialBalance}`)
    log(`- constructor | 💵 HOPR Balance: ${this.initialHoprBalance}`)
    log(`- constructor | 🐛 Debug Mode: ${COVERBOT_DEBUG_MODE}`)
    log(`- constructor | ✅ Verification Engaged: ${COVERBOT_VERIFICATION_CYCLE_EXECUTE}`)
    log(`- constructor | 👀 Verification Cycle: ${COVERBOT_VERIFICATION_CYCLE_IN_MS}`)
    log(`- constructor | 🔍 Relaying Cycle: ${RELAY_VERIFICATION_CYCLE_IN_MS}`)

    this.ethereumAddress = null
    this.chainId = null
    this.network = null

    this.xdaiWeb3 = new Web3(new Web3.providers.HttpProvider(COVERBOT_CHAIN_PROVIDER))
    this.verificationTimeout =
      COVERBOT_VERIFICATION_CYCLE_EXECUTE &&
      setInterval(this._startCycles.bind(this), COVERBOT_VERIFICATION_CYCLE_IN_MS)

    this.verifiedHoprNodes = new Map<string, HoprNode>()
    this.relayTimeouts = new Map<string, NodeJS.Timeout>()
    this._loadData().catch((err) => error(`- constructor | Initial data load failed.`, err))
  }

  protected _loadData() {
    return loadData.call(this)
  }

  protected _dumpData() {
    return dumpData.call(this)
  }

  protected _startCycles() {
    coverTrafficCycle.call(this)
  }

  protected _sendMessageFromBot(recipient: string, message: string, intermediatePeerIds = [], includeRecipient = true) {
    log(`- sendMessageFromBot | Sending ${intermediatePeerIds.length} hop message to ${recipient}`)
    log(`- sendMessageFromBot | Message: ${message}`)
    return this.node.send({
      peerId: recipient,
      payload: message,
      intermediatePeerIds,
      includeRecipient,
    })
  }

  protected async _verifyBalance(message: IMessage): Promise<[number, NodeStates]> {
    const pubkey = await convertPubKeyFromB58String(message.from)
    const nodeEthereumAddress = u8aToHex(await Utils.pubKeyToAccountId(pubkey.marshal()))
    const weiBalance = await this.xdaiWeb3.eth.getBalance(nodeEthereumAddress)
    const balance = +Web3.utils.fromWei(weiBalance)

    return balance >= COVERBOT_XDAI_THRESHOLD
      ? [balance, NodeStates.xdaiBalanceSucceeded]
      : [balance, NodeStates.xdaiBalanceFailed]
  }

  protected async _verifyTweet(message: IMessage): Promise<[TweetMessage, VerifyTweetStates]> {
    try {
      log(`- _verifyTweet | Obtained message: ${message}`)
      const tweet = new TweetMessage(message.text)
      log(`- _verifyTweet | Valid tweet with id ${tweet.id} and url ${tweet.url}. Proceeding to fetch content.`)
      await tweet.fetch({ mock: COVERBOT_DEBUG_MODE })
      log(`- _verifyTweet | Successfully fetch content for tweet created at ${tweet.created_at}`)
      this.tweets.set(message.from, tweet)
      log(`- _verifyTweet | Updated our tweets map. Now we have ${this.tweets.size} tweets in memory.`)
      log(`- _verifyTweet | Starting validation process for the tweet with id ${tweet.id}.`)
      if (tweet.hasTag('hoprnetwork')) {
        log(`- _verifyTweet | Tweet ${tweet.id} has tag #HOPRNetwork.`)
        tweet.status.hasTag = true
      }
      if (tweet.hasMention('hoprnet')) {
        log(`- _verifyTweet | Tweet ${tweet.id} has mentioned @hoprnet.`)
        tweet.status.hasMention = true
      }
      if (tweet.hasSameHOPRNode(message.from) || COVERBOT_DEBUG_MODE) {
        log(`- _verifyTweet | Tweet ${tweet.id} comes from the same node that is messaging me.`)
        tweet.status.sameNode = true
      }
      log(`- _verifyTweet | Completed validation process for the tweet with id ${tweet.id}.`)
      COVERBOT_DEBUG_MODE && log(`- _verifyTweet | DEBUG_MODE enabled, automatically validating tweet.`)
      COVERBOT_DEBUG_MODE && tweet.validateTweetStatus()
      const tweetIsValid = tweet.status.isValid()
      log(`- _verifyTweet | The tweet is considered ${tweetIsValid ? 'valid' : 'invalid'}. Returning result.`)
      return tweetIsValid
        ? [tweet, VerifyTweetStates.tweetVerificationSucceeded]
        : [tweet, VerifyTweetStates.tweetVerificationFailed]
    } catch (err) {
      error(`- _verifyTweet | Error when trying to verify tweet: ${message}`, err)
      return [undefined, VerifyTweetStates.tweetInvalid]
    }
  }

  async handleMessage(message: IMessage) {
    log(`- handleMessage | ${this.botName} <- ${message.from}: ${message.text}`)

    const instructionQueue = message.text.split(' ')
    const maybeCommand = instructionQueue.shift()

    try {
      const instructionWrapper = new Instruction(maybeCommand)
      while (instructionQueue.length > 0) {
        instructionWrapper.enterInput(instructionQueue.shift())
      }

      switch (instructionWrapper.command) {
        case BotCommands.verify: {
          log(`- handleMessage | ${BotCommands.verify} command received`)
          verificatorReducer.call(this, instructionWrapper, message)
          break
        }
        case BotCommands.stats: {
          log(`- handleMessage | ${BotCommands.stats} command received`)
          statsReducer.call(this, instructionWrapper, message)
          break
        }
        case BotCommands.admin: {
          log(`- handleMessage | ${BotCommands.admin} command received`)
          adminReducer.call(this, instructionWrapper, message)
          break
        }
        case BotCommands.rules: {
          log(`- handleMessage | ${BotCommands.rules} command received`)
          rulesReducer.call(this, message)
          break
        }

        default: {
          log(`- handleMessage | Command was not understood`)
          helpReducer.call(this, message)
          break
        }
      }
    } catch (err) {
      log(`- handleMessage | Instruction was not understood`)
      helpReducer.call(this, message)
      return
    }
  }
}
