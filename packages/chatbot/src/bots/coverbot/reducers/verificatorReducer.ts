import Instruction from "../classes/instruction";
import { VerifySubCommands, BotCommands } from "../types/commands";
import { VerifyTweetStates } from "../types/states";
import { VerifyStateResponses, VerifyTweetStateResponse } from "../responses";
import { IMessage } from "src/message/message";
import debug from 'debug'
import { Coverbot } from "..";


const log = debug('hopr-chatbot:reducers:verificator')
const error = debug('hopr-chatbot:reducers:verificator:error')

export async function verificatorReducer(this: Coverbot, instructionWrapper: Instruction, message: IMessage) {
  log(`- verificatorReducer | Starting verificatorReducer with instruction ${instructionWrapper.toString()} and message ${message.text} from ${message.from}`)
  switch (instructionWrapper.subcommand) {
    case VerifySubCommands.tweet: {
      log(`- verificatorReducer | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand received`)
      const maybeTweetUrl = instructionWrapper.content;
      log(`- verificatorReducer | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand :: Received maybeTweet with content ${maybeTweetUrl}`)
      try {
        if (instructionWrapper.content.match(/https:\/\/twitter.com.*?$/i)) {
          log(`- verificatorReducer | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand :: The content ${maybeTweetUrl} can be considered a tweet, but we are yet verify it’s valid`)
          const [tweet, verificationState] = await this._verifyTweet(message)

          if (!tweet) {
            error(`- verificatorReducer | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand :: The tweet is invalid or other error was found. Telling user about it.`)
            this._sendMessageFromBot(message.from, ((VerifyTweetStateResponse[VerifyTweetStates.tweetInvalid] as Function)(instructionWrapper.content))).catch((err) => {
              error(`Trying to send ${VerifyTweetStates.tweetVerificationSucceeded} message to ${message.from} failed.`, err)
            })
          }

          log(`- verificatorReducer | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand :: The tweet is valid, and comes from ${tweet.screen_name} and created on ${tweet.created_at}.`)
          log(`- verificatorReducer | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand :: Informing the user whether their tweet passed verification: ${verificationState}.`)

          if (verificationState === VerifyTweetStates.tweetVerificationSucceeded) {
            log(`- verificatorReducer | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand :: The tweet passed the verification, informing user.`)
            this._sendMessageFromBot(message.from, (VerifyTweetStateResponse[VerifyTweetStates.tweetVerificationSucceeded])).catch((err) => {
              error(`Trying to send ${VerifyTweetStates.tweetVerificationSucceeded} message to ${message.from} failed.`, err)
            })

            // Validating balance of node process goes here.
            log(`- verificatorReducer | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand :: @TODO: Verifying user balance.`)

          } else {
            log(`- verificatorReducer | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand :: The tweet failed the verification, informing user.`)
            this._sendMessageFromBot(message.from, (VerifyTweetStateResponse[VerifyTweetStates.tweetVerificationFailed] as Function)(tweet)).catch((err) => {
              error(`Trying to send ${VerifyTweetStates.tweetVerificationFailed} message to ${message.from} failed.`, err)
            })
          }

          log(`- verificatorReducer | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand :: Completing verification process. Ending verification reduce.`)
          return

        } else {
          error(`- verificatorReducer | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand :: The content doesn’t contain a tweet URL is invalid or other error was found. Telling user about it.`)
          this._sendMessageFromBot(message.from, ((VerifyTweetStateResponse[VerifyTweetStates.tweetInvalid] as Function)(instructionWrapper.content))).catch((err) => {
            error(`Trying to send ${VerifyTweetStates.tweetVerificationSucceeded} message to ${message.from} failed.`, err)
          })
          return
        }
      } catch (err) {
        error(`Trying to parse ${maybeTweetUrl} failed at some point.`, err)
        this._sendMessageFromBot(message.from, VerifyTweetStates[VerifyTweetStates.tweetInvalid]).catch((err) => {
          error(`Trying to send ${BotCommands.stats} message to ${message.from} failed.`, err)
        })
        return
      }
      this._sendMessageFromBot(message.from, (VerifyStateResponses[VerifySubCommands.tweet] as Function)(maybeTweetUrl)).catch((err) => {
        error(`Trying to send ${BotCommands.stats} message to ${message.from} failed.`, err)
      })
      break;
    }
    default: {
      log(`- verificatorReducer | ${BotCommands.stats} command :: subcommand not understood`)
      this._sendMessageFromBot(message.from, VerifyStateResponses[VerifySubCommands.help] as string).catch((err) => {
        error(`Trying to send ${VerifySubCommands.help} message to ${message.from} failed.`, err)
      })
    }
  }
}