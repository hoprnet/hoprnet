import type API from '../utils/api'
import sinon from 'sinon'
import {
  shouldFailExecutionOnInvalidQuery,
  shouldFailExecutionOnInvalidParam,
  shouldFailExecutionOnApiError,
  shouldSucceedExecution
} from './behaviours.spec'
import Withdraw from './withdraw'
import { NATIVE_ADDRESS, PEER_A } from '../utils/fixtures'

type Response = Awaited<ReturnType<API['withdraw']>>

const createCommand = (withdrawResponse: Response, getCachedAliasesResponse?: Record<any, any> | undefined) => {
  const api = sinon.fake() as unknown as API
  api.withdraw = () => Promise.resolve(withdrawResponse)
  const cache = {
    getCachedAliases: () => getCachedAliasesResponse || ({} as Record<any, any>)
  }

  return new Withdraw(api, cache as any)
}

describe('test Withdraw command', function () {
  const cmdWithOkRes = createCommand({
    ok: true,
    json: async () => ({
      receipt: '0xreceipt'
    })
  } as Response)
  const cmdWithBadRes = createCommand({
    ok: false
  } as Response)

  shouldFailExecutionOnInvalidQuery(cmdWithOkRes, 'x')
  shouldFailExecutionOnInvalidQuery(cmdWithOkRes, 'x x')
  shouldFailExecutionOnInvalidParam(cmdWithOkRes, `1 ETH ${NATIVE_ADDRESS}`)
  shouldFailExecutionOnInvalidParam(cmdWithOkRes, `ETH 1 ${NATIVE_ADDRESS}`)
  shouldFailExecutionOnInvalidParam(cmdWithOkRes, `1 hopr ${PEER_A}`)
  shouldFailExecutionOnApiError(cmdWithBadRes, `1 hopr ${NATIVE_ADDRESS}`)
  shouldSucceedExecution(cmdWithOkRes, [`1 hopr ${NATIVE_ADDRESS}`, ['Withdrawing', 'Withdrawed']])
  shouldSucceedExecution(cmdWithOkRes, [`1 HOPR ${NATIVE_ADDRESS}`, ['Withdrawing', 'Withdrawed']])
  shouldSucceedExecution(cmdWithOkRes, [`1 native ${NATIVE_ADDRESS}`, ['Withdrawing', 'Withdrawed']])
  shouldSucceedExecution(cmdWithOkRes, [`1 NATIVE ${NATIVE_ADDRESS}`, ['Withdrawing', 'Withdrawed']])
})
