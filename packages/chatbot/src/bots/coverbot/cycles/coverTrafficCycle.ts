import debug from 'debug'
import { COVERBOT_VERIFICATION_CYCLE_IN_MS, COVERBOT_DEBUG_HOPR_ADDRESS, COVERBOT_DEBUG_MODE } from '../../../utils/env'
import { HoprNode } from '../types/coverbot'
import { TweetMessage } from '../../../lib/twitter/twitter'
import { BotResponses, NodeStateResponses } from '../responses'
import { BotCommands } from '../types/commands'
import { NodeStates } from '../types/states'
import { RELAY_VERIFICATION_CYCLE_IN_MS, databaseTextRef } from '../utils/constants'
import { Coverbot } from '..'


const log = debug('hopr-chatbot:coverTraffic')
const error = debug('hopr-chatbot:coverTraffic:error')

export async function coverTrafficCycle(this: Coverbot) {
  if (!this.loadedDb) {
    log(`- coverTrafficCycle | Database ${databaseTextRef} isn‘t loaded, calling this.loadData`)
    await this._loadData()
  }

  log(`- coverTrafficCycle | ${COVERBOT_VERIFICATION_CYCLE_IN_MS}ms has passed. Verifying nodes...`)
  COVERBOT_DEBUG_MODE && log(`- coverTrafficCycle | DEBUG mode activated, looking for ${COVERBOT_DEBUG_HOPR_ADDRESS}`)

  log(`- coverTrafficCycle | Getting ready to dump data.`)
  await this._dumpData()
  log(`- coverTrafficCycle | Dump data process completed.`)

  const _verifiedNodes = Array.from(this.verifiedHoprNodes.values())
  log(`- coverTrafficCycle | ${_verifiedNodes.length} verified nodes read from memory.`)
  const randomIndex = Math.floor(Math.random() * _verifiedNodes.length)
  log(`- coverTrafficCycle | Random index ${randomIndex} picked to choose a verified node.`)
  const hoprNode: HoprNode = _verifiedNodes[randomIndex]
  log(`- coverTrafficCycle | Node ${hoprNode} selected at random to go through verification.`)

  if (!hoprNode) {
    log(`- coverTrafficCycle | No node from our memory. Skipping...`)
    return
  }

  if (this.relayTimeouts.get(hoprNode.id)) {
    log(`- coverTrafficCycle | Node ${hoprNode.id} selected is going through relaying. Skipping...`)
    return
  }

  try {
    log(`- coverTrafficCycle | Verifying node process, looking for tweet ${hoprNode.tweetUrl}.`)
    const tweet = new TweetMessage(hoprNode.tweetUrl)
    await tweet.fetch({ mock: COVERBOT_DEBUG_MODE })
    log(`- coverTrafficCycle | Verifying node process, tweet section. About to get HOPR node from Tweet.`)
    const _hoprNodeAddress = tweet.getHOPRNode({ mock: COVERBOT_DEBUG_MODE, hoprNode: COVERBOT_DEBUG_HOPR_ADDRESS })
    log(`- coverTrafficCycle | Verifying node process, tweet section. HOPR Node ${_hoprNodeAddress} found.`)

    if (_hoprNodeAddress.length === 0) {
      log(`- coverTrafficCycle | No node has been found from our tweet w/content ${tweet.content}`)
      // this.verifiedHoprNodes.delete(hoprNode.id)
      await this._dumpData()
    } else {
      log(`- coverTrafficCycle | Node ${_hoprNodeAddress} has been found. Trying to notify verification process.`)
      this._sendMessageFromBot(_hoprNodeAddress, BotResponses[BotCommands.verify]).catch((err) => {
        error(`Trying to send ${BotCommands.verify} message to ${_hoprNodeAddress} failed.`, err)
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
      log(`- coverTrafficCycle | Relaying node ${_hoprNodeAddress}, checking in ${RELAY_VERIFICATION_CYCLE_IN_MS}`)
      log(`- coverTrafficCycle | Sending 0 hop messages to ${_hoprNodeAddress} to notify relaying process.`)
      this._sendMessageFromBot(_hoprNodeAddress, NodeStateResponses[NodeStates.onlineNode]).catch((err) => {
        error(`Trying to send ${NodeStates.onlineNode} message to ${_hoprNodeAddress} failed.`, err)
      })

      // 2.
      log(`- coverTrafficCycle | Sending multihop messages to ${_hoprNodeAddress} to start process.`)
      this._sendMessageFromBot(this.address, `verify relay ${_hoprNodeAddress}`, [_hoprNodeAddress]).catch(
        (err) => {
          error(`Trying to send RELAY message to ${_hoprNodeAddress} failed.`, err)
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
          log(`- coverTrafficCycle | No response from ${_hoprNodeAddress}.`) // Removing as valid node.`)

          // 4.1.2
          log(`- coverTrafficCycle | Sending 0 hop message to ${_hoprNodeAddress} to notify invalid relay`)
          this._sendMessageFromBot(_hoprNodeAddress, NodeStateResponses[NodeStates.relayingNodeFailed]).catch(
            (err) => {
              error(`Trying to send ${NodeStates.relayingNodeFailed} message to ${_hoprNodeAddress} failed.`, err)
            },
          )

          // 4.1.3
          log(`- coverTrafficCycle | Removing expired timeout for ${_hoprNodeAddress} from memory.`)
          this.relayTimeouts.delete(_hoprNodeAddress)

          // 4.1.4
          //this.verifiedHoprNodes.delete(_hoprNodeAddress)
          //this.dumpData()
        }, RELAY_VERIFICATION_CYCLE_IN_MS),
      )
    }
  } catch (err) {
    console.log('[ _coverTrafficCycle ] Error caught - ', err)

    // Something failed. We better remove node and update.
    // @TODO: Clean this up, removed for now to ask users to try again.
    // this.verifiedHoprNodes.delete(hoprNode.id)
    // this.dumpData()
  }
}