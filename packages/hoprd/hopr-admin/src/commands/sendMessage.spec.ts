import type API from '../utils/api'
import sinon from 'sinon'
import {
  shouldFailExecutionOnInvalidQuery,
  shouldFailExecutionOnApiError,
  shouldSucceedExecution,
  shouldFailExecution
} from './behaviours.spec'
import SendMessage from './sendMessage'
import { PEER_A as HOP_1, PEER_B as HOP_2, PEER_C as RECIPIENT } from '../utils/fixtures'

type Response = Awaited<ReturnType<API['sendMessage']>>

const BODY = 'hello world'

const createCommand = (sendMessageResponse: Response, getCachedAliasesResponse?: Record<any, any> | undefined) => {
  const api = sinon.fake() as unknown as API
  api.sendMessage = () => Promise.resolve(sendMessageResponse)
  api.getAddresses = () =>
    Promise.resolve({
      ok: true,
      json: async () => ({
        hopr: 'SELF'
      })
    } as Response)
  api.getSettings = () =>
    Promise.resolve({
      ok: true,
      json: async () => ({
        includeRecipient: false
      })
    } as Response)
  const cache = {
    getCachedAliases: () => getCachedAliasesResponse || ({} as Record<any, any>)
  }

  return new SendMessage(api, cache as any)
}

describe('test SendMessage command', function () {
  const cmdWithOkApiAuto = createCommand({
    ok: true,
    json: async () => ({
      body: BODY,
      recipient: RECIPIENT
    })
  } as Response)
  const cmdWithOkApiDirect = createCommand({
    ok: true,
    json: async () => ({
      body: BODY,
      recipient: RECIPIENT,
      path: []
    })
  } as Response)
  const cmdWithOkApiManual = createCommand({
    ok: true,
    json: async () => ({
      body: BODY,
      recipient: RECIPIENT,
      path: [HOP_1, HOP_2]
    })
  } as Response)
  const cmdWithBadRes = createCommand({
    ok: false
  } as Response)

  shouldFailExecutionOnInvalidQuery(cmdWithOkApiAuto, 'x')
  shouldFailExecutionOnApiError(cmdWithBadRes, `${RECIPIENT} hello`)
  shouldSucceedExecution(cmdWithOkApiAuto, [
    `${RECIPIENT} hello world 1 2 3`,
    [`Sending message to ${RECIPIENT} using automatic path finding ..`, `Message to ${RECIPIENT} sent`]
  ])
  shouldSucceedExecution(cmdWithOkApiDirect, [
    `,${RECIPIENT} hello directly`,
    [`Sending direct message to ${RECIPIENT} ..`, `Message to ${RECIPIENT} sent`]
  ])
  shouldSucceedExecution(cmdWithOkApiManual, [
    `${HOP_1},${HOP_2},${RECIPIENT} hello manually`,
    [`Sending message to ${RECIPIENT} via ${HOP_1}->${HOP_2} ..`, `Message to ${RECIPIENT} sent`]
  ])
  shouldFailExecution(cmdWithBadRes, [`${HOP_1},${HOP_1},${RECIPIENT} hello manually`, 'to construct path'])
})
