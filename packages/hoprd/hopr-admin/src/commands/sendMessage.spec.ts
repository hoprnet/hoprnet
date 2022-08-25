import type API from '../utils/api'
import sinon from 'sinon'
// import { shouldBehaveLikeACommand } from './behaviours.spec'
import SendMessage from './sendMessage'

type Response = Awaited<ReturnType<API['sendMessage']>>

const BODY = 'hello world'
const HOP_1 = '16Uiu2HAm1uV82HyD1iJ5DmwJr4LftmJUeMfj8zFypBRACmrJc16n'
const RECIPIENT = '16Uiu2HAm2SF8EdwwUaaSoYTiZSddnG4hLVF7dizh32QFTNWMic2b'

const createCommand = (sendMessageResponse: Response, getCachedAliasesResponse?: Record<any, any> | undefined) => {
  const api = sinon.fake() as unknown as API
  api.sendMessage = () => Promise.resolve(sendMessageResponse)
  const cache = {
    getCachedAliases: () => getCachedAliasesResponse || ({} as Record<any, any>)
  }

  return new SendMessage(api, cache as any)
}

describe('test SendMessage command', function () {
  const cmdWithApi = createCommand({
    ok: true,
    json: async () => ({
      body: BODY,
      recipient: RECIPIENT,
      path: [HOP_1]
    })
  } as Response)
  const cmdWithNoApi = createCommand({
    ok: false
  } as Response)

  // shouldBehaveLikeACommand(
  //   cmdWithApi,
  //   cmdWithNoApi,
  //   [
  //     [`${RECIPIENT} hello`, [`Sending message to ${RECIPIENT} using automatic path finding ...`]], // automatic path finding
  //     [`,${RECIPIENT} hello`, [`Sending direct message to ${RECIPIENT} ...`]], // direct message
  //     [`${HOP_1},${RECIPIENT} hello`, [`Sending message to ${RECIPIENT} via ${HOP_1} ...`]] // manual path
  //   ],
  //   []
  // )
})
