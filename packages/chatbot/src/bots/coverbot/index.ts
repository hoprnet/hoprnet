import Web3 from 'web3'
import { Bot } from '../bot'
import { IMessage } from '../../message/message'
import { TweetMessage } from '../../lib/twitter/twitter'
import FirebaseDatabase from '../../lib/firebase/api'
//@TODO: Isolate these utilities to avoid importing the entire package
import { convertPubKeyFromB58String, u8aToHex } from '@hoprnet/hopr-utils'
import { Utils } from '@hoprnet/hopr-core-ethereum'
import { Networks, HOPR_CHANNELS } from '@hoprnet/hopr-core-ethereum/lib/ethereum/addresses'
import {
  COVERBOT_DEBUG_MODE,
  COVERBOT_CHAIN_PROVIDER,
  COVERBOT_VERIFICATION_CYCLE_IN_MS,
  COVERBOT_XDAI_THRESHOLD,
  COVERBOT_VERIFICATION_CYCLE_EXECUTE,
  HOPR_ENVIRONMENT,
  COVERBOT_DEBUG_HOPR_ADDRESS,
  FIREBASE_DATABASE_URL,
  COVERBOT_ADMIN_MODE
} from '../../utils/env'
import db from './db'
import { NodeStates, VerifyTweetStates } from './types/states'
import { RELAY_VERIFICATION_CYCLE_IN_MS } from './constants'
import { BotResponses, NodeStateResponses, AdminStateResponses, StatsStateResponses, VerifyStateResponses } from './responses'
import { BalancedHoprNode, HoprNode } from './types/coverbot'
import Core from '../../lib/hopr/core'
import Instruction from './instruction'
import { BotCommands, VerifySubCommands, StatsSubCommands, AdminSubCommands } from './types/commands'
import debug from 'debug'
import { verificatorReducer } from './reducers/verificatorReducer'


const log = debug('hopr-chatbot:coverbot')
const error = debug('hopr-chatbot:coverbot:error')
const { fromWei } = Web3.utils

const scoreDbRef = db.ref(`/${HOPR_ENVIRONMENT}/score`)
const stateDbRef = db.ref(`/${HOPR_ENVIRONMENT}/state`)
const databaseTextRef = `${FIREBASE_DATABASE_URL} @ Table ${HOPR_ENVIRONMENT}`

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
  network: Networks
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
    this.verificationTimeout = COVERBOT_VERIFICATION_CYCLE_EXECUTE && setInterval(this._verificationCycle.bind(this), COVERBOT_VERIFICATION_CYCLE_IN_MS)

    this.verifiedHoprNodes = new Map<string, HoprNode>()
    this.relayTimeouts = new Map<string, NodeJS.Timeout>()
    this.loadData()
      .catch(err => error(`- constructor | Initial data load failed.`, err))
  }

  private async _getEthereumAddressFromHOPRAddress(hoprAddress: string): Promise<string> {
    const pubkey = await convertPubKeyFromB58String(hoprAddress)
    const ethereumAddress = u8aToHex(await Utils.pubKeyToAccountId(pubkey.marshal()))
    return ethereumAddress
  }

  private async _getEthereumAddressScore(ethereumAddress: string): Promise<number> {
    return new Promise((resolve, reject) => {
      scoreDbRef.child(ethereumAddress).once('value', (snapshot, error) => {
        if (error) return reject(error)
        return resolve(snapshot.val() || 0)
      })
    })
  }

  private async _setEthereumAddressScore(ethereumAddress: string, score: number): Promise<void> {
    return new Promise((resolve, reject) => {
      scoreDbRef.child(ethereumAddress).setWithPriority(score, -score, (error) => {
        if (error) return reject(error)
        return resolve()
      })
    })
  }

  private async loadData(): Promise<void> {
    log(`- loadData | Loading data from Database (${databaseTextRef})`)
    return new Promise((resolve, reject) => {
      this.database.getSchema(HOPR_ENVIRONMENT).then(snapshot => {
        if (!snapshot) {
          log(`- loadData | Database (${databaseTextRef}) hasn’t been created`)
          return resolve()
        }
        const { state } = snapshot || {};
        log(`- loadData | State ${JSON.stringify(state)} obtained from database`)
        const connected = state && state.connected ? state.connected : []
        log(`- loadData | Loaded ${connected.length} nodes from our Database (${databaseTextRef})`)
        this.verifiedHoprNodes = this.verifiedHoprNodes.values.length > 0 ? this.verifiedHoprNodes : new Map<string, HoprNode>()
        connected.forEach((n) => this.verifiedHoprNodes.set(n.id, n))
        log(`- loadData | Updated ${Array.from(this.verifiedHoprNodes.values()).length} verified nodes in memory`)

        this.loadedDb = true
        return resolve()
      }).catch(err => {
        error(`- loadData | Error retrieving data`, err)
        if (error) return reject(error)
      })
    })
  }

  protected async dumpData() {
    log(`- dumpData | Starting dumping data in Database (${databaseTextRef})`)
    //@TODO: Ideally we move this to a more suitable place.
    if (!this.ethereumAddress) {
      this.chainId = await Utils.getChainId(this.xdaiWeb3)
      this.network = Utils.getNetworkName(this.chainId) as Networks
      this.ethereumAddress = await this._getEthereumAddressFromHOPRAddress(this.address)
    }

    const connectedNodes = this.node.listConnectedPeers()
    log(`- loadData | Detected ${connectedNodes} in the network w/bootstrap servers ${this.node.getBootstrapServers()}`)

    const state = {
      connectedNodes,
      env: {
        COVERBOT_CHAIN_PROVIDER,
        COVERBOT_DEBUG_MODE,
        COVERBOT_VERIFICATION_CYCLE_IN_MS,
        COVERBOT_XDAI_THRESHOLD,
      },
      hoprCoverbotAddress: await this._getEthereumAddressFromHOPRAddress(this.address),
      hoprChannelContract: HOPR_CHANNELS[this.network],
      address: this.address,
      balance: fromWei(await this.xdaiWeb3.eth.getBalance(this.ethereumAddress)),
      available: fromWei(await this.node.getHoprBalance()),
      locked: 0, //@TODO: Retrieve balances from open channels.
      connected: Array.from(this.verifiedHoprNodes.values()),
      refreshed: new Date().toISOString(),
    }

    return new Promise((resolve, reject) => {
      stateDbRef.set(state, (error) => {
        if (error) return reject(error)
        log(`- dumpData | Saved data in Database (${databaseTextRef})`)
        return resolve()
      })
    })
  }

  protected _sendMessageFromBot(recipient:string, message: string, intermediatePeerIds = [], includeRecipient = true) {
    log(`- sendMessageFromBot | Sending ${intermediatePeerIds.length} hop message to ${recipient}`)
    log(`- sendMessageFromBot | Message: ${message}`)
    return this.node.send({
      peerId: recipient,
      payload: message,
      intermediatePeerIds,
      includeRecipient,
    })
  }

  protected async _verificationCycle() {
    if (!this.loadedDb) {
      log(`- verificationCycle | Database ${databaseTextRef} isn‘t loaded, calling this.loadData`)
      await this.loadData()
    }

    log(`- verificationCycle | ${COVERBOT_VERIFICATION_CYCLE_IN_MS}ms has passed. Verifying nodes...`)
    COVERBOT_DEBUG_MODE && log(`- verificationCycle | DEBUG mode activated, looking for ${COVERBOT_DEBUG_HOPR_ADDRESS}`)

    log(`- verificationCycle | Getting ready to dump data.`)
    await this.dumpData()
    log(`- verificationCycle | Dump data process completed.`)

    const _verifiedNodes = Array.from(this.verifiedHoprNodes.values())
    log(`- verificationCycle | ${_verifiedNodes.length} verified nodes read from memory.`)
    const randomIndex = Math.floor(Math.random() * _verifiedNodes.length)
    log(`- verificationCycle | Random index ${randomIndex} picked to choose a verified node.`)
    const hoprNode: HoprNode = _verifiedNodes[randomIndex]
    log(`- verificationCycle | Node ${hoprNode} selected at random to go through verification.`)

    if (!hoprNode) {
      log(`- verificationCycle | No node from our memory. Skipping...`)
      return
    }

    if (this.relayTimeouts.get(hoprNode.id)) {
      log(`- verificationCycle | Node ${hoprNode.id} selected is going through relaying. Skipping...`)
      return
    }

    try {
      log(`- verificationCycle | Verifying node process, looking for tweet ${hoprNode.tweetUrl}.`)
      const tweet = new TweetMessage(hoprNode.tweetUrl)
      await tweet.fetch({ mock: COVERBOT_DEBUG_MODE })
      log(`- verificationCycle | Verifying node process, tweet section. About to get HOPR node from Tweet.`)
      const _hoprNodeAddress = tweet.getHOPRNode({ mock: COVERBOT_DEBUG_MODE, hoprNode: COVERBOT_DEBUG_HOPR_ADDRESS })
      log(`- verificationCycle | Verifying node process, tweet section. HOPR Node ${_hoprNodeAddress} found.`)

      if (_hoprNodeAddress.length === 0) {
        log(`- verificationCycle | No node has been found from our tweet w/content ${tweet.content}`)
        // this.verifiedHoprNodes.delete(hoprNode.id)
        await this.dumpData()
      } else {
        log(`- verificationCycle | Node ${_hoprNodeAddress} has been found. Trying to notify verification process.`)
        this._sendMessageFromBot(_hoprNodeAddress, BotResponses[BotCommands.verify]).catch((err) => {
          error(`Trying to send ${BotCommands.verify} message to ${_hoprNodeAddress} failed.`)
        })
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
        log(`- verificationCycle | Relaying node ${_hoprNodeAddress}, checking in ${RELAY_VERIFICATION_CYCLE_IN_MS}`)
        log(`- verificationCycle | Sending 0 hop messages to ${_hoprNodeAddress} to notify relaying process.`)
        this._sendMessageFromBot(_hoprNodeAddress, NodeStateResponses[NodeStates.onlineNode]).catch((err) => {
          error(`Trying to send ${NodeStates.onlineNode} message to ${_hoprNodeAddress} failed.`)
        })

        // 2.
        log(`- verificationCycle | Sending multihop messages to ${_hoprNodeAddress} to start process.`)
        this._sendMessageFromBot(this.address, `verify relay ${_hoprNodeAddress}`, [_hoprNodeAddress]).catch(
          (err) => {
            error(`Trying to send RELAY message to ${_hoprNodeAddress} failed.`)
          },
        )

        // 3.
        this.relayTimeouts.set(
          _hoprNodeAddress,
          setTimeout(() => {
            // 4.1
            /*
             * The timeout passed, and we didn‘t get the message back.
             * 4.1.1 Internally log that this is the case.
             * 4.1.2 Let the node that we couldn't get our response back in time.
             * 4.1.3 Remove from timeout so they can try again somehow.
             * NB: DELETED BY PB AFTER CHAT 10/9 [4.1.4 Remove from our verified node and write to the database]
             */

            // 4.1.1
            log(`- verificationCycle | No response from ${_hoprNodeAddress}.`) // Removing as valid node.`)

            // 4.1.2
            log(`- verificationCycle | Sending 0 hop message to ${_hoprNodeAddress} to notify invalid relay`)
            this._sendMessageFromBot(_hoprNodeAddress, NodeStateResponses[NodeStates.relayingNodeFailed]).catch(
              (err) => {
                error(`Trying to send ${NodeStates.relayingNodeFailed} message to ${_hoprNodeAddress} failed.`)
              },
            )

            // 4.1.3
            log(`- verificationCycle | Removing expired timeout for ${_hoprNodeAddress} from memory.`)
            this.relayTimeouts.delete(_hoprNodeAddress)

            // 4.1.4
            //this.verifiedHoprNodes.delete(_hoprNodeAddress)
            //this.dumpData()
          }, RELAY_VERIFICATION_CYCLE_IN_MS),
        )
      }
    } catch (err) {
      console.log('[ _verificationCycle ] Error caught - ', err)

      // Something failed. We better remove node and update.
      // @TODO: Clean this up, removed for now to ask users to try again.
      // this.verifiedHoprNodes.delete(hoprNode.id)
      // this.dumpData()
    }
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
    //@TODO: Catch error here.
    const tweet = new TweetMessage(message.text)
    this.tweets.set(message.from, tweet)

    await tweet.fetch({ mock: COVERBOT_DEBUG_MODE })

    if (tweet.hasTag('hoprnetwork')) {
      tweet.status.hasTag = true
    }
    if (tweet.hasMention('hoprnet')) {
      tweet.status.hasMention = true
    }
    if (tweet.hasSameHOPRNode(message.from) || COVERBOT_DEBUG_MODE) {
      tweet.status.sameNode = true
    }

    COVERBOT_DEBUG_MODE && tweet.validateTweetStatus()

    return tweet.status.isValid()
      ? [tweet, VerifyTweetStates.tweetVerificationSucceeded]
      : [tweet, VerifyTweetStates.tweetVerificationFailed]
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
          log(`- handleMessage | verify command received`)
          verificatorReducer(instructionWrapper, message)
          break;
        }
        case BotCommands.stats: {
          log(`- handleMessage | ${BotCommands.stats} command received`)
          const snapshot = (await this.database.getSchema(HOPR_ENVIRONMENT)) || {};
          log(`- handleMessage | ${BotCommands.stats} command :: retrieving snapshot with value ${JSON.stringify(snapshot)}`)
          const state = snapshot && snapshot.state ? snapshot.state : {};
          log(`- handleMessage | ${BotCommands.stats} command :: retrieving state with value ${JSON.stringify(state)}`)
          switch (instructionWrapper.subcommand) {
            case StatsSubCommands.connected: {
              log(`- handleMessage | ${BotCommands.stats} command :: ${StatsSubCommands.connected} subcommand received`)
              log(`- handleMessage | ${BotCommands.stats} command :: ${StatsSubCommands.connected} subcommand retrieving state from snapshot with value ${state}`)
              const connectedNodes:number = state && state.connectedNodes ? state.connectedNodes : 0
              log(`- handleMessage | ${BotCommands.stats} command :: ${StatsSubCommands.connected} subcommand retrieving connected nodes from state with value ${connectedNodes}`)
              this._sendMessageFromBot(message.from, (StatsStateResponses[StatsSubCommands.connected] as Function)(connectedNodes) ).catch((err) => {
                error(`Trying to send ${BotCommands.stats} message to ${message.from} failed.`)
              })
              break;
            }
            default: {
              log(`- handleMessage | ${BotCommands.stats} command :: subcommand not understood`)
              this._sendMessageFromBot(message.from, StatsStateResponses[StatsSubCommands.help] as string).catch((err) => {
                error(`Trying to send ${StatsSubCommands.help} message to ${message.from} failed.`)
              })
            }
          }
          break;
        }
        case BotCommands.admin: {
          log(`- handleMessage | ${BotCommands.admin} command received`)
          if (!COVERBOT_ADMIN_MODE) {
            return this._sendMessageFromBot(message.from, NodeStateResponses[NodeStates.adminModeDisabled]).catch((err) => {
              error(`Trying to send ${NodeStates.adminModeDisabled} message to ${message.from} failed.`)
            })
          } else {
            log(`- handleMessage | ${BotCommands.admin} command :: allowed to go forward`)
            switch (instructionWrapper.subcommand) {
              case AdminSubCommands.help: {
                log(`- handleMessage | ${BotCommands.admin} command :: ${AdminSubCommands.help} subcommand received`)
                this._sendMessageFromBot(message.from, AdminStateResponses[AdminSubCommands.help]).catch((err) => {
                  error(`Trying to send ${AdminSubCommands.help} message to ${message.from} failed.`)
                })
                break;
              }
              case AdminSubCommands.verificationCycle: {
                log(`- handleMessage | ${BotCommands.admin} command :: ${AdminSubCommands.verificationCycle} subcommand received`)
                this._sendMessageFromBot(message.from, AdminStateResponses[AdminSubCommands.verificationCycle]).catch((err) => {
                  error(`Trying to send ${AdminSubCommands.verificationCycle} message to ${message.from} failed.`)
                })
                log(`- handleMessage | ${BotCommands.admin} command :: ${AdminSubCommands.verificationCycle} subcommand :: starting verification cycle`)
                await this._verificationCycle.call(this);
                log(`- handleMessage | ${BotCommands.admin} command :: ${AdminSubCommands.verificationCycle} subcommand :: completed verification cycle`)
                break;
              }
              case AdminSubCommands.saveState: {
                log(`- handleMessage | ${BotCommands.admin} command :: ${AdminSubCommands.saveState} subcommand received`)
                this._sendMessageFromBot(message.from, AdminStateResponses[AdminSubCommands.saveState]).catch((err) => {
                  error(`Trying to send ${AdminSubCommands.saveState} message to ${message.from} failed.`)
                })
                log(`- handleMessage | ${BotCommands.admin} command :: ${AdminSubCommands.saveState} subcommand :: starting saving state`)
                await this.dumpData()
                log(`- handleMessage | ${BotCommands.admin} command :: ${AdminSubCommands.saveState} subcommand :: completed saving state`)
                break;
              }
              default: {
                log(`- handleMessage | admin command :: subcommand not understood`)
                this._sendMessageFromBot(message.from, AdminStateResponses[AdminSubCommands.help]).catch((err) => {
                  error(`Trying to send ${AdminSubCommands.help} message to ${message.from} failed.`)
                })
              }
            }

          }
          break;
        }
        case BotCommands.rules: {
          log(`- handleMessage | rules command received`)
          this._sendMessageFromBot(message.from, BotResponses[BotCommands.rules]).catch((err) => {
            error(`Trying to send ${BotCommands.rules} message to ${message.from} failed.`)
          })
          break;
        }

        default: {
          log(`- handleMessage | Command was not understood`)
          this._sendMessageFromBot(message.from, BotResponses[BotCommands.help]).catch((err) => {
            error(`Trying to send ${BotCommands.help} message to ${message.from} failed.`)
          })
        }
      }

    } catch (err) {
      log(`- handleMessage | Instruction was not understood`)
      this._sendMessageFromBot(message.from, BotResponses[BotCommands.help]).catch((err) => {
        error(`Trying to send ${BotCommands.help} message to ${message.from} failed.`)
      })
      return;
    }
  }
}
