import type API from '../utils/api'
import sinon from 'sinon'
import {
  shouldFailExecutionOnInvalidQuery,
  shouldFailExecutionOnInvalidParam,
  shouldFailExecutionOnApiError,
  shouldSucceedExecution
} from './behaviours.spec'
import Ping from './ping'
import { PEER_A } from '../utils/fixtures'

type Response = Awaited<ReturnType<API['ping']>>

const createCommand = (pingResponse: Response, getCachedAliasesResponse?: Record<any, any> | undefined) => {
  const api = sinon.fake() as unknown as API
  api.ping = () => Promise.resolve(pingResponse)
  const cache = {
    getCachedAliases: () => getCachedAliasesResponse || ({} as Record<any, any>)
  }

  return new Ping(api, cache as any)
}

describe('test Ping command', function () {
  const cmdWithOkRes = createCommand({
    ok: true,
    json: async () => ({
      latency: 100
    })
  } as Response)
  const cmdWithBadRes = createCommand({
    ok: false
  } as Response)

  shouldFailExecutionOnInvalidQuery(cmdWithOkRes, 'x x')
  shouldFailExecutionOnInvalidParam(cmdWithOkRes, '1')
  shouldFailExecutionOnApiError(cmdWithBadRes, PEER_A)
  shouldSucceedExecution(cmdWithOkRes, [PEER_A, [`Pong from peer ${PEER_A} received in 100 ms`]])
})
