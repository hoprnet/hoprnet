import type API from '../utils/api'
import sinon from 'sinon'
import {
  shouldFailExecutionOnInvalidQuery,
  shouldFailExecutionOnInvalidParam,
  shouldFailExecutionOnApiError,
  shouldSucceedExecution
} from './behaviours.spec'
import Balances from './balances'

type Response = Awaited<ReturnType<API['getBalances']>>

const createCommand = (getResponse: Response, getCachedAliasesResponse?: Record<any, any> | undefined) => {
  const api = sinon.fake() as unknown as API
  api.getBalances = () => Promise.resolve(getResponse)
  const cache = {
    getCachedAliases: () => getCachedAliasesResponse || ({} as Record<any, any>),
    getSymbols: () => ({
      native: 'native',
      hopr: 'hopr',
      nativeDisplay: `NATIVE (native)`,
      hoprDisplay: `HOPR (hopr)`
    })
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

  shouldFailExecutionOnInvalidQuery(cmdWithOkApi, 'x x x')
  shouldFailExecutionOnInvalidParam(cmdWithOkApi, '1')
  shouldFailExecutionOnApiError(cmdWithBadApi, '')
  shouldSucceedExecution(cmdWithOkApi, ['', ['HOPR (hopr) Balance:']])
  shouldSucceedExecution(cmdWithOkApi, ['native', ['1']])
  shouldSucceedExecution(cmdWithOkApi, ['hopr', ['2']])
})
