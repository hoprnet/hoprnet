import type API from '../utils/api'
import sinon from 'sinon'
import * as behaviours from './behaviours.spec'
import Balances from './balances'

type Response = Awaited<ReturnType<API['getBalances']>>

const createCommand = (getResponse: Response, getCachedAliasesResponse?: Record<any, any> | undefined) => {
  const api = sinon.fake() as unknown as API
  api.getBalances = () => Promise.resolve(getResponse)
  const cache = {
    getCachedAliases: () => getCachedAliasesResponse || ({} as Record<any, any>)
  }

  return new Balances(api, cache as any)
}

describe('test Balances command', function () {
  const cmdWithOkApi = createCommand({
    ok: true,
    json: async () => ({
      native: '1',
      hopr: '2'
    })
  } as Response)
  const cmdWithBadApi = createCommand({
    ok: false
  } as Response)

  behaviours.shouldFailExecutionOnInvalidQuery(cmdWithOkApi, 'x x x')
  behaviours.shouldFailExecutionOnInvalidParam(cmdWithOkApi, '1')
  behaviours.shouldFailExecutionOnApiError(cmdWithBadApi, '')
  behaviours.shouldSucceedExecution(cmdWithOkApi, ['', ['HOPR Balance:']])
  behaviours.shouldSucceedExecution(cmdWithOkApi, ['native', ['1']])
  behaviours.shouldSucceedExecution(cmdWithOkApi, ['hopr', ['2']])
})
