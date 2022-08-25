import type API from '../utils/api'
import sinon from 'sinon'
// import { shouldBehaveLikeACommand } from './behaviours.spec'
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
  const cmdWithApi = createAddressesCommand({
    ok: true,
    json: async () => ({
      native: 'NATIVE_ADDRESS_MOCK',
      hopr: 'HOPR_ADDRESS_MOCK'
    })
  } as GetAddressesResponse)
  const cmdWithNoApi = createAddressesCommand({
    ok: false
  } as GetAddressesResponse)

  // shouldBehaveLikeACommand(
  //   cmdWithApi,
  //   cmdWithNoApi,
  //   [
  //     ['', ['HOPR_ADDRESS_MOCK']],
  //     ['hopr', ['HOPR_ADDRESS_MOCK']],
  //     ['native', ['NATIVE_ADDRESS_MOCK']]
  //   ],
  //   []
  // )
})
