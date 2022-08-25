import type API from '../utils/api'
import sinon from 'sinon'
import * as behaviours from './behaviours.spec'
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

describe.only('test Ping command', function () {
  const cmdWithOkRes = createCommand({
    ok: true,
    json: async () => ({
      latency: 100
    })
  } as Response)
  const cmdWithBadRes = createCommand({
    ok: false
  } as Response)

  behaviours.shouldFailExecutionOnInvalidQuery(cmdWithOkRes, 'x x')
  behaviours.shouldFailExecutionOnIncorrectParam(cmdWithOkRes, '1')
  behaviours.shouldFailExecutionOnApiError(cmdWithBadRes, PEER_A)
  behaviours.shouldSucceedExecution(cmdWithOkRes, [[PEER_A, ['latency']]])
})
