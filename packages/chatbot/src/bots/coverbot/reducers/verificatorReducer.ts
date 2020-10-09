import Instruction from "../classes/instruction";
import { VerifySubCommands, BotCommands } from "../types/commands";
import { VerifyTweetStates } from "../types/states";
import { VerifyStateResponses } from "../responses";
import { IMessage } from "src/message/message";
import debug from 'debug'
import { Coverbot } from "..";


const log = debug('hopr-chatbot:reducers:verificator')
const error = debug('hopr-chatbot:reducers:verificator:error')

export function verificatorReducer(this: Coverbot, instructionWrapper: Instruction, message: IMessage) {
  log(`- verificatorReducer | Starting verificatorReducer with instruction ${instructionWrapper.toString()} and message ${message.text} from ${message.from}`)
  switch (instructionWrapper.subcommand) {
    case VerifySubCommands.tweet: {
      log(`- verificatorReducer | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand received`)

      //VerificationAction to be done here.

      const maybeTweetUrl = instructionWrapper.content;
      log(`- verificatorReducer | ${BotCommands.verify} command :: ${VerifySubCommands.tweet} subcommand :: Received maybeTweet with content ${maybeTweetUrl}`)

      try {

      } catch (err) {
        error(`Trying to parse ${maybeTweetUrl} failed.`, err)
        this._sendMessageFromBot(message.from, VerifyTweetStates[VerifyTweetStates.tweetInvalid]).catch((err) => {
          error(`Trying to send ${BotCommands.stats} message to ${message.from} failed.`, err)
        })
        return;
      }

      this._sendMessageFromBot(message.from, VerifyStateResponses[VerifySubCommands.tweet] as string).catch((err) => {
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