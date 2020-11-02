import { COVERBOT_XDAI_THRESHOLD } from '../../utils/env'
import { RELAY_VERIFICATION_CYCLE_IN_MS, ScoreRewards } from './utils/constants'
import { NodeStates, VerifyTweetStates } from './types/states'
import { BotCommands, AdminSubCommands, StatsSubCommands, VerifySubCommands } from './types/commands'
import { TweetState, TweetMessage } from '../../lib/twitter/twitter'

type BotResponse = {
  [key in BotCommands]: string
}

type AdminResponse = {
  [key in AdminSubCommands]: string
}

type VerifyResponse = {
  [key in VerifySubCommands]: string | Function
}

type StatsResponse = {
  [key in StatsSubCommands]: string | Function
}

export type GenericResponse = BotResponse | AdminResponse | VerifyResponse | StatsResponse

export const AdminStateResponses: AdminResponse = {
  [AdminSubCommands.help]: `\n
    You are using the super admin command. Please run one of the following.

    admin coverTrafficCycle   - Starts a single verification cycle on all verified nodes.
    admin saveState           - Saves current state of the bot into the database.
    admin help                - Shows you this message again.
  `,
  [AdminSubCommands.coverTrafficCycle]: `\n
    Starting manually verification cycle. Please review the logs to see the
    changes done against the system state and database.
  `,
  [AdminSubCommands.saveState]: `\n
    Starting manually saving state to database. Please review the logs to see the
    changes done against the system state and database.
  `
}

export const StatsStateResponses: StatsResponse = {
  [StatsSubCommands.help]: `\n
    You are using the stats command. Please run one of the following.

    stats connected   - Shows you the amount of connected nodes in the network.
    stats help        - Shows you this message again.
  `,
  [StatsSubCommands.connected]: (connected: number) => `\n
    There are currently ${connected} nodes connected in the network.
  `,
}

export const VerifyStateResponses: VerifyResponse = {
  [VerifySubCommands.tweet]: (maybeTweet: string) => `\n
    You have requested to verify the tweet "${maybeTweet}". Please wait while
    we fetch its content and verify your node against it.

    Expect a few messages from our side. If you want to know the status of
    your node, please send "verify status $YOUR_NODE_ADRESS".
  `,
  [VerifySubCommands.help]: `\n
    You are using the verify command. Please run one of the following.

    verify tweet  $tweet_link    - Sends a tweet to verify your node in the network.
    verify status $hopr_address  - Request information about your verification status.
    verify help                  - Shows you this message again.
  `,
  [VerifySubCommands.status]: (status: NodeStates) => `\n
    Your current verification status is: ${status}
  `
}

export const VerifyTweetStateResponse = {
  [VerifyTweetStates.tweetInvalid]: (maybeTweet: string) => `\n
    You passed ${maybeTweet}, but it isn’t a valid tweet. Please try again
    providing a tweet URL in the form https://twitter.com/$user/status/$id.

    e.g. verify tweet https://twitter.com/jjperezaguinaga/status/1311330405375762433
    `,
  [VerifyTweetStates.tweetVerificationFailed]: (tweetStatus: TweetState) => `\n
    Your tweet has failed the verification. Please make sure you've included everything.

    Here is the current status of your tweet:
    1. Tagged @hoprnet: ${tweetStatus.hasMention}
    2. Used #HOPRNetwork: ${tweetStatus.hasTag}
    3. Includes this node address: ${tweetStatus.sameNode}

    Please try again with a different tweet.
  `,
  [VerifyTweetStates.tweetVerificationSucceeded]: `\n
    Your tweet has passed verification. Please do no delete this tweet, as I'll
    use it multiple times to verify and connect to your node.

    I’ll now check that your HOPR Ethereum address has at least ${COVERBOT_XDAI_THRESHOLD} xDAI.
    If you need xDAI, you always swap DAI to xDAI using https://dai-bridge.poa.network/.
  `,
}

export const BotResponses: BotResponse = {
  [BotCommands.rules]: `\n
    Welcome to the HOPR incentivized network!

    1. Load ${COVERBOT_XDAI_THRESHOLD} xDAI into your HOPR Ethereum Address
    2. Post a tweet with your HOPR Address and the tag #HOPRNetwork
    3. Send me the link to your tweet (don't delete it!)
    4. Every time you're chosen to relay a message, you'll score ${ScoreRewards.relayed} points and receive HOPR!

    Thank you for powering the HOPR Network.
  `,
  [BotCommands.help]: `\n
    Hi! My name is coverbot. Please tell me how I can help you by sending a
    message with the following command:

    verify  - Start the verification process for cover traffic
    stats   - Learn about the current stats for the network
    rules   - Learn the rules about the incentivation network
    help    - Show you this message again
    
    You can get also request help for each command. E.g verify help.
  `,
  [BotCommands.stats]: StatsStateResponses[StatsSubCommands.help] as string,
  [BotCommands.verify]: VerifyStateResponses[VerifySubCommands.help] as string,
  [BotCommands.admin]: AdminStateResponses[AdminSubCommands.help]
}

export const NodeStateResponses = {
  [NodeStates.adminModeDisabled]: `\n
    You are trying to use an admin command, but the admin mode hasn’t been
    enabled for this session. Sorry!
  `,
  [NodeStates.newUnverifiedNode]: BotResponses[BotCommands.rules],
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
    and pay you in xHOPR tokens for your service.

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
