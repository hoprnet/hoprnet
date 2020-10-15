import { TweetState } from '../../lib/twitter/twitter'
import { COVERBOT_XDAI_THRESHOLD } from '../../utils/env'
import { RELAY_VERIFICATION_CYCLE_IN_MS } from './constants'
import { BotCommands, NodeStates, ScoreRewards } from './state'

export const BotResponses = {
  [BotCommands.rules]: `\n
    Welcome to the HOPR incentivized network!

    1. Load ${COVERBOT_XDAI_THRESHOLD} xDAI into your HOPR Ethereum Address
    2. Post a tweet with your HOPR Address and the tag #HOPRNetwork
    3. Send me the link to your tweet (don't delete it!)
    4. Every time you're chosen to relay a message, you'll score ${ScoreRewards.relayed} points and receive HOPR!

    Visit https://saentis.hoprnet.org for more information and scoreboard
  `,
  [BotCommands.status]: (status: NodeStates) => `\n
    Your current status is: ${status}
  `,
  [BotCommands.verify]: `\n
    Verifying if your node is still up...
  `,
}

export const NodeStateResponses = {
  [NodeStates.newUnverifiedNode]: BotResponses[BotCommands.rules],
  [NodeStates.tweetVerificationFailed]: (tweetStatus: TweetState) => `\n
      Your tweet has failed the verification. Please make sure you've included everything.
  
      Here is the current status of your tweet:
      1. Tagged @hoprnet: ${tweetStatus.hasMention}
      2. Used #HOPRNetwork: ${tweetStatus.hasTag}
      3. Includes this node address: ${tweetStatus.sameNode}
  
      Please try again with a different tweet.
    `,
  [NodeStates.tweetVerificationSucceeded]: `\n
      Your tweet has passed verification. Please do no delete this tweet, as I'll
      use it multiple times to verify and connect to your node.
  
      I’ll now check that your HOPR Ethereum address has at least ${COVERBOT_XDAI_THRESHOLD} xDAI.
      If you need xDAI, you always swap DAI to xDAI using https://dai-bridge.poa.network/.
    `,
  [NodeStates.tweetVerificationInProgress]: `\n
      Thank you for your Tweet! I‘ll now try to verify it...
    `,
  [NodeStates.xdaiBalanceFailed]: (xDaiBalance: number) => `\n
      Your node does not have at least ${COVERBOT_XDAI_THRESHOLD} xDAI. Currently, your node has ${xDaiBalance} xDAI.
  
      To participate in our incentivized network, please send the missing amount of xDAI to your node.
    `,
  [NodeStates.xdaiBalanceSucceeded]: (xDaiBalance: number) => `\n
      Your node has ${xDaiBalance} xDAI. You're ready to go!
  
      Soon I'll open a payment channel to your node.
  
      Please keep your balance topped up, and your tweet and node online.
  
      For more information, go to https://saentis.hoprnet.org
  
      Thank you for participating in our incentivized network!
    `,
  [NodeStates.onlineNode]: `\n
      Node online! Relaying a message to verify your ability to
      send messages to other nodes in the network.
    `,
  [NodeStates.verifiedNode]: `\n
      Verification successful! I’ll shortly use you as a cover traffic node
      and pay you in HOPR tokens for your service.
  
      For more information, go to https://saentis.hoprnet.org
  
      Thank you for participating in our incentivized network!
    `,
  [NodeStates.relayingNodeFailed]: `\n
      Relaying failed. I can reach you, but you can’t reach me...
      
      This could mean other errors though, so please keep trying.
  
      For more information, go to https://saentis.hoprnet.org
  
      Visit our Telegram (https://t.me/hoprnet) for any questions.
    `,
  [NodeStates.relayingNodeInProgress]: `\n
      Relaying in progress. I've received your message and I'm now
      waiting for a packet I'm trying to relay via your node. Please wait
      until I receive your packet or ${RELAY_VERIFICATION_CYCLE_IN_MS / 1000}seconds
      elapse. Thank you for your patience!
    `,
  [NodeStates.relayingNodeSucceded]: `\n
      Relaying successful! I've obtained a packet anonymously from you,
      and we can move to the next verification step.
    `,
}

export const VERIFY_MESSAGE = `\n
  
`
