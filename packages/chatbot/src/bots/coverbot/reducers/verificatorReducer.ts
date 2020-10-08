import Instruction from "../instruction";
import { VerifySubCommands, BotCommands } from "../types/commands";
import { VerifyTweetStates } from "../types/states";
import { VerifyStateResponses } from "../responses";
import { IMessage } from "src/message/message";
import debug from 'debug'


const log = debug('hopr-chatbot:reducers:verificator')
const error = debug('hopr-chatbot:reducers:verificator:error')

export function verificatorReducer(instructionWrapper: Instruction, message: IMessage) {
  switch (instructionWrapper.subcommand) {
    case VerifySubCommands.tweet: {
      log(`- handleMessage | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand received`)

      //VerificationAction to be done here.

      const maybeTweetUrl = instructionWrapper.content;
      log(`- handleMessage | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand :: Received maybeTweet with content ${maybeTweetUrl}`)

      try {

      } catch (err) {
        error(`Trying to parse ${maybeTweetUrl} failed.`, err)
        this._sendMessageFromBot(message.from, VerifyTweetStates[VerifyTweetStates.tweetInvalid]).catch((err) => {
          error(`Trying to send ${BotCommands.stats} message to ${message.from} failed.`)
        })
        return;
      }

      this._sendMessageFromBot(message.from, VerifyStateResponses[VerifySubCommands.tweet] as string).catch((err) => {
        error(`Trying to send ${BotCommands.stats} message to ${message.from} failed.`)
      })

      break;
    }
    default: {
      log(`- handleMessage | ${BotCommands.stats} command :: subcommand not understood`)
      this._sendMessageFromBot(message.from, VerifyStateResponses[VerifySubCommands.help] as string).catch((err) => {
        error(`Trying to send ${VerifySubCommands.help} message to ${message.from} failed.`)
      })
    }
  }
}