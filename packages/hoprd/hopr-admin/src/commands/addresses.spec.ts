import type API from '../utils/api'
import sinon from 'sinon'
import {
  shouldFailExecutionOnInvalidQuery,
  shouldFailExecutionOnInvalidParam,
  shouldFailExecutionOnApiError,
  shouldSucceedExecution
} from './behaviours.spec'
import Addresses from './addresses'

type GetAddressesResponse = Awaited<ReturnType<API['getAddresses']>>

const createAddressesCommand = (
  getAddressesResponse: GetAddressesResponse,
  getCachedAliasesResponse?: Record<any, any> | undefined
) => {
  const api = sinon.fake() as unknown as API
  api.getAddresses = () => Promise.resolve(getAddressesResponse)
  const cache = {
    getCachedAliases: () => getCachedAliasesResponse || ({} as Record<any, any>),
    getSymbols: () => ({
      native: 'native',
      hopr: 'hopr',
      nativeDisplay: `NATIVE (native)`,
      hoprDisplay: `HOPR (hopr)`
    })
  }

  return new Addresses(api, cache as any)
}

describe('test Addresses command', function () {
  const cmdWithOkApi = createAddressesCommand({
    ok: true,
    json: async () => ({
      native: 'NATIVE_ADDRESS_MOCK',
      hopr: 'HOPR_ADDRESS_MOCK'
    })
  } as GetAddressesResponse)
  const cmdWithBadApi = createAddressesCommand({
    ok: false
  } as GetAddressesResponse)

  shouldFailExecutionOnInvalidQuery(cmdWithOkApi, 'x x x')
  shouldFailExecutionOnInvalidParam(cmdWithOkApi, '1')
  shouldFailExecutionOnApiError(cmdWithBadApi, '')
  shouldSucceedExecution(cmdWithOkApi, ['', ['HOPR (hopr) Address:']])
  shouldSucceedExecution(cmdWithOkApi, ['native', ['NATIVE_ADDRESS_MOCK']])
  shouldSucceedExecution(cmdWithOkApi, ['hopr', ['HOPR_ADDRESS_MOCK']])
})
