import type API from '../utils/api'
import sinon from 'sinon'
import { shouldBehaveLikeACommand } from './behaviours.spec'
import Ping from './ping'

type PingResponse = Awaited<ReturnType<API['ping']>>

const createAddressesCommand = (
  pingResponse: PingResponse,
  getCachedAliasesResponse?: Record<any, any> | undefined
) => {
  const api = sinon.fake() as unknown as API
  api.ping = () => Promise.resolve(pingResponse)
  const cache = {
    getCachedAliases: () => getCachedAliasesResponse || ({} as Record<any, any>)
  }

  return new Ping(api, cache as any)
}

describe('test Ping command', function () {
  const cmdWithApi = createAddressesCommand({
    ok: true,
    json: async () => ({
      latecny: 100
    })
  } as PingResponse)
  const cmdWithNoApi = createAddressesCommand({
    ok: false
  } as PingResponse)

  shouldBehaveLikeACommand(
    cmdWithApi,
    cmdWithNoApi,
    'INVALID',
    '',
    [
      ['', ['HOPR_ADDRESS_MOCK']],
      ['hopr', ['HOPR_ADDRESS_MOCK']],
      ['native', ['NATIVE_ADDRESS_MOCK']]
    ],
    []
  )
})
