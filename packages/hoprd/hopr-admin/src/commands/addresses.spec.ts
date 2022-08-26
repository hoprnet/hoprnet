import type API from '../utils/api'
import sinon from 'sinon'
import * as behaviours from './behaviours.spec'
import Addresses from './addresses'

type GetAddressesResponse = Awaited<ReturnType<API['getAddresses']>>

const createAddressesCommand = (
  getAddressesResponse: GetAddressesResponse,
  getCachedAliasesResponse?: Record<any, any> | undefined
) => {
  const api = sinon.fake() as unknown as API
  api.getAddresses = () => Promise.resolve(getAddressesResponse)
  const cache = {
    getCachedAliases: () => getCachedAliasesResponse || ({} as Record<any, any>)
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

  behaviours.shouldFailExecutionOnInvalidQuery(cmdWithOkApi, 'x x x')
  behaviours.shouldFailExecutionOnInvalidParam(cmdWithOkApi, '1')
  behaviours.shouldFailExecutionOnApiError(cmdWithBadApi, '')
  behaviours.shouldSucceedExecution(cmdWithOkApi, ['', ['HOPR Address:']])
  behaviours.shouldSucceedExecution(cmdWithOkApi, ['native', ['NATIVE_ADDRESS_MOCK']])
  behaviours.shouldSucceedExecution(cmdWithOkApi, ['hopr', ['HOPR_ADDRESS_MOCK']])
})
