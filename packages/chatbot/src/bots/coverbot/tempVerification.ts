        // if (message.from === this.address) {
        //     /*
        //     * We have done a succeful round trip!
        //     * 1. Lets avoid sending more messages to eternally loop
        //     *    messages across the network by returning within this if.
        //     * 2. Let's notify the user about the successful relay.
        //     * 3. Let's recover the timeout from our relayerTimeout
        //     *    and clear it before it removes the node.
        //     * 4. Let's pay the good node some sweet xHOPR for being alive
        //     *    and relaying messages successfully.
        //     */
        //     const relayerAddress = getHOPRNodeAddressFromContent(instructionWrapper.content)

        //     // 2.
        //     log(`- handleMessage | Successful Relay: ${relayerAddress}`)
        //     this._sendMessageFromBot(relayerAddress, NodeStateResponses[NodeStates.relayingNodeSucceded]).catch((err) => {
        //       error(`Trying to send ${NodeStates.relayingNodeSucceded} message to ${relayerAddress} failed.`)
        //     })

        //     // 3.
        //     const relayerTimeout = this.relayTimeouts.get(relayerAddress)
        //     clearTimeout(relayerTimeout)
        //     this.relayTimeouts.delete(relayerAddress)

        //     // 4.
        //     const relayerEthereumAddress = await this._getEthereumAddressFromHOPRAddress(relayerAddress)
        //     const score = await this._getEthereumAddressScore(relayerEthereumAddress)
        //     const newScore = score + ScoreRewards.relayed

        //     await Promise.all([
        //       this._setEthereumAddressScore(relayerEthereumAddress, newScore),
        //       this.node.withdraw({ currency: 'HOPR', recipient: relayerEthereumAddress, amount: `${RELAY_HOPR_REWARD}` }),
        //     ])
        //     console.log(`xHOPR tokens sent to ${relayerAddress}`)
        //     this._sendMessageFromBot(relayerAddress, NodeStateResponses[NodeStates.verifiedNode])

        //     // 1.
        //     return
        //   }

        //   if (this.relayTimeouts.get(message.from)) {
        //     /*
        //     * There‘s a particular case where someone can send us a message while
        //     * we are trying to relay them a package. We'll skip the entire process
        //     * if this is the case, as the timeout will clear them out.
        //     *
        //     * 1. Detect if we have someone waiting for timeout (this if).
        //     * 2. If so, then quickly return them a message telling we are waiting.
        //     * 3. Return as to avoid going through the entire process again.
        //     *
        //     */

        //     // 2.
        //     this._sendMessageFromBot(message.from, NodeStateResponses[NodeStates.relayingNodeInProgress]).catch((err) => {
        //       error(`Trying to send ${NodeStates.relayingNodeInProgress} message to ${message.from} failed.`)
        //     })

        //     // 3.
        //     return
        //   }

        

        //   switch (nodeState) {
        //     case NodeStates.newUnverifiedNode:
        //       this._sendMessageFromBot(message.from, NodeStateResponses[nodeState]).catch((err) => {
        //         error(`Trying to send ${nodeState} message to ${message.from} failed.`)
        //       })
        //       break
        //     case NodeStates.tweetVerificationFailed:
        //       this._sendMessageFromBot(
        //         message.from,
        //         NodeStateResponses[nodeState](this.tweets.get(message.from).status),
        //       ).catch((err) => {
        //         error(`Trying to send ${nodeState} message to ${message.from} failed.`)
        //       })
        //       break
        //     case NodeStates.tweetVerificationSucceeded:
        //       this._sendMessageFromBot(message.from, NodeStateResponses[nodeState]).catch((err) => {
        //         error(`Trying to send ${nodeState} message to ${message.from} failed.`)
        //       })
        //       const [balance, xDaiBalanceNodeState] = await this._verifyBalance(message)
        //       switch (xDaiBalanceNodeState) {
        //         case NodeStates.xdaiBalanceFailed:
        //           this._sendMessageFromBot(message.from, NodeStateResponses[xDaiBalanceNodeState](balance)).catch((err) => {
        //             error(`Trying to send ${xDaiBalanceNodeState} message to ${message.from} failed.`)
        //           })
        //           break
        //         case NodeStates.xdaiBalanceSucceeded: {
        //           const ethAddress = await this._getEthereumAddressFromHOPRAddress(message.from)

        //           this.verifiedHoprNodes.set(message.from, {
        //             id: message.from,
        //             tweetId: tweet.id,
        //             tweetUrl: tweet.url,
        //             address: ethAddress,
        //           })

        //           const score = await this._getEthereumAddressScore(ethAddress)
        //           if (score === 0) {
        //             await this._setEthereumAddressScore(ethAddress, ScoreRewards.verified)
        //           }

        //           this._sendMessageFromBot(message.from, NodeStateResponses[xDaiBalanceNodeState](balance)).catch((err) => {
        //             error(`Trying to send ${xDaiBalanceNodeState} message to ${message.from} failed.`)
        //           })
        //           break
        //         }
        //       }
        //       this._sendMessageFromBot(message.from, BotResponses[VerifySubCommands.status](xDaiBalanceNodeState)).catch((err) => {
        //         error(`Trying to send ${VerifySubCommands.status} message to ${message.from} failed.`)
        //       })
        //       break
        //   }
        //   this._sendMessageFromBot(message.from, BotResponses[VerifySubCommands.status](nodeState)).catch((err) => {
        //     error(`Trying to send ${VerifySubCommands.status} message to ${message.from} failed.`)
        //   })